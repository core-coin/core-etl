use async_recursion::async_recursion;
use atoms_rpc_types::BlockNumberOrTag;
use config::Config;
use futures::stream::StreamExt;
use provider::Provider;
use std::error::Error;
use storage::Storage;
use tracing::info;

pub struct ETLWorker {
    pub config: Config,
    pub storage: Box<dyn Storage + Sync>,
    pub provider: Provider,
    // pub newest_block: i64,
}

impl ETLWorker {
    pub fn new(
        config: Config,
        storage: Box<dyn Storage + Sync + Send + 'static>,
        provider: Provider,
    ) -> Self {
        Self {
            config,
            storage,
            provider,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        info!("ETLWorker is running");
        // let block = self
        //     .provider
        //     .get_block(BlockNumberOrTag::Latest)
        //     .await
        //     .unwrap();
        // println!("Block unwrapped: {:#?}", block.clone());

        // self.storage.add_block(block.clone()).await?;

        // let block = self.storage.get_block_by_number(block.number).await?;
        // println!("Block DB: {:#?}", block);
        self.sync_old_blocks().await?;
        info!("Stale blocks syncing is finished");
        self.sync_new_blocks().await?;
        Ok(())
    }

    pub async fn sync_new_blocks(&mut self) -> Result<(), Box<dyn Error>> {
        info!("Syncing new blocks");
        let subscription = self.provider.subscribe_blocks().await;
        let mut stream = subscription
            .into_stream()
            .take_while(|x| futures::future::ready(x.header.number.is_some()));

        while let Some(block) = stream.next().await {
            info!("Imported new block: {:?}", block.header.number.unwrap());
            self.storage.add_block(block.into()).await?;
        }

        Ok(())
    }

    #[async_recursion]
    pub async fn sync_old_blocks(&mut self) -> Result<(), Box<dyn Error>> {
        let mut synced = 0;

        let latest_chain = self
            .provider
            .get_block(BlockNumberOrTag::Latest)
            .await
            .unwrap();

        let mut latest_db = self.storage.get_latest_block_number().await.unwrap();
        if latest_chain.number == latest_db {
            return Ok(());
        }
        info!(
            "Syncing stale blocks from {} to {}",
            latest_db, latest_chain.number
        );
        loop {
            if latest_chain.number == latest_db {
                info!("DB is synced on block {}", latest_chain.number);
                self.sync_old_blocks().await?;
                break;
            }
            latest_db += 1;
            synced += 1;
            let new_block = self
                .provider
                .get_block(BlockNumberOrTag::Number(latest_db as u64))
                .await
                .unwrap();
            self.storage.add_block(new_block).await?;

            if synced % 1000 == 0 {
                info!("Synced {} blocks", synced);
            }
        }

        Ok(())
    }

    pub async fn update_blocks_to_matured(
        &mut self,
        block_height: i64,
    ) -> Result<(), Box<dyn Error>> {
        self.storage.update_blocks_to_matured(block_height).await?;
        Ok(())
    }
}
