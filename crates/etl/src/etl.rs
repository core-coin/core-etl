use atoms_rpc_types::{Block as AtomsBlock, BlockNumberOrTag};
use config::Config;
use futures::stream::StreamExt;
use provider::Provider;
use std::{error::Error, sync::Arc};
use storage::Storage;
use tokio::sync::Mutex;
use tokio::{spawn, sync::MutexGuard};
use tracing::{error, info};
use types::{Block, Transaction};

pub struct ETLWorker {
    pub config: Config,
    storage: Arc<Mutex<dyn Storage>>,
    provider: Provider,
    // pub newest_block: i64,
}

// Clone here makes a copy of the Arc pointer - not  the entire class of data
// All clones point to the same internal data
impl Clone for ETLWorker {
    fn clone(&self) -> Self {
        ETLWorker {
            storage: Arc::clone(&self.storage),
            config: self.config.clone(),
            provider: self.provider.clone(),
        }
    }
}

impl ETLWorker {
    pub fn new(config: Config, storage: Arc<Mutex<dyn Storage>>, provider: Provider) -> Self {
        Self {
            config,
            storage,
            provider,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        info!("ETLWorker is running");
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

        // Add data about the new block to the database
        // At first we get header via stream, then we get block and transactions and store them
        while let Some(header) = stream.next().await {
            let block_height = header.header.number.unwrap().clone() as i64;
            let (block, transactions) = self
                .provider
                .get_block_with_transactions(BlockNumberOrTag::Number(block_height as u64))
                .await
                .unwrap();
            info!(
                "Imported new block {:?} with {:?} transactions",
                block.number,
                transactions.len()
            );
            // Add block to the database
            self.get_safe_storage()
                .await
                .add_block(block.clone().into())
                .await?;
            // Add transactions to the database
            self.get_safe_storage()
                .await
                .add_transactions(transactions)
                .await?;

            // Update blocks to matured
            let mut clone = self.clone();
            spawn(async move {
                if let Ok(()) = clone.update_blocks_to_matured(block_height - 5).await {
                    info!("Blocks till {:?} are matured", block_height - 5);
                } else {
                    error!("Failed to mature blocks");
                }
            });
        }

        Ok(())
    }

    pub async fn sync_old_blocks(&mut self) -> Result<(), Box<dyn Error>> {
        Box::pin(async move {
            let mut synced = 0;

            // Get the latest block from the chain
            let latest_chain = self
                .provider_get_block(BlockNumberOrTag::Latest)
                .await
                .unwrap();

            // Update blocks to matured
            let mut clone = self.clone();
            let s = spawn(async move {
                if let Ok(()) = clone
                    .update_blocks_to_matured(latest_chain.number - 5)
                    .await
                {
                    info!("Blocks till {:?} are matured", latest_chain.number - 5);
                } else {
                    error!("Failed to mature blocks");
                }
            });

            // Get the latest block from the database
            let mut latest_db = self.storage_get_latest_block_number().await.unwrap();
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
                // Get the next block from the chain and add it to the database
                let (new_block, new_txs) = self
                    .provider_get_block_with_transactions(BlockNumberOrTag::Number(
                        latest_db as u64,
                    ))
                    .await
                    .unwrap();
                self.get_safe_storage().await.add_block(new_block).await?;
                self.get_safe_storage()
                    .await
                    .add_transactions(new_txs)
                    .await?;

                if synced % 1000 == 0 {
                    info!("Synced {} blocks", synced);
                }
            }

            tokio::join!(s).0?;

            Ok(())
        })
        .await
    }

    pub async fn update_blocks_to_matured(
        &mut self,
        block_height: i64,
    ) -> Result<(), Box<dyn Error>> {
        self.storage
            .lock()
            .await
            .update_blocks_to_matured(block_height)
            .await?;
        Ok(())
    }
}

impl ETLWorker {
    async fn storage_get_latest_block_number(&self) -> Result<i64, Box<dyn Error>> {
        self.storage.lock().await.get_latest_block_number().await
    }

    async fn get_safe_storage(&self) -> MutexGuard<dyn Storage> {
        self.storage.lock().await
    }

    async fn provider_get_block(&self, block: BlockNumberOrTag) -> Result<Block, Box<dyn Error>> {
        let block = self.provider.get_block(block).await;
        Ok(block.unwrap())
    }

    async fn provider_get_block_with_transactions(
        &self,
        block: BlockNumberOrTag,
    ) -> Result<(Block, Vec<Transaction>), Box<dyn Error>> {
        let res = self.provider.get_block_with_transactions(block).await;
        Ok(res.unwrap())
    }
}
