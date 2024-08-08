use atoms_rpc_types::BlockNumberOrTag;
use config::Config;
use futures::future::join_all;
use futures::stream::StreamExt;
use provider::Provider;
use std::pin::Pin;
use std::{error::Error, sync::Arc};
use storage::Storage;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::{spawn, sync::MutexGuard};
use tracing::{error, info};
use types::{Block, SyncMode, Transaction};

pub struct ETLWorker {
    pub config: Config,
    storage: Arc<Mutex<dyn Storage>>,
    provider: Provider,

    sync_mode: SyncMode,
}

// Clone here makes a copy of the Arc pointer - not  the entire class of data
// All clones point to the same internal data
impl Clone for ETLWorker {
    fn clone(&self) -> Self {
        ETLWorker {
            storage: Arc::clone(&self.storage),
            config: self.config.clone(),
            provider: self.provider.clone(),
            sync_mode: self.sync_mode.clone(),
        }
    }
}

impl ETLWorker {
    pub async fn new(config: Config, storage: Arc<Mutex<dyn Storage>>, provider: Provider) -> Self {
        let mut etl = ETLWorker {
            config,
            storage,
            provider,
            sync_mode: SyncMode::FromZeroBlock,
        };

        if etl
            .storage
            .lock()
            .await
            .get_latest_block_number()
            .await
            .unwrap()
            != 0
            || etl.config.continue_sync
        // if we have some blocks in the database - we need to continue syncing
        {
            etl.sync_mode = SyncMode::FromLastBlockInDB;
            return etl;
        }
        if etl.config.block_number != 0 {
            etl.sync_mode = SyncMode::FromBlock(etl.config.block_number);
        }
        etl
    }

    pub async fn run(&mut self) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        info!("ETLWorker is running");
        self.sync_old_blocks().await?;
        info!("Stale blocks syncing is finished");
        self.sync_new_blocks().await?;
        Ok(())
    }

    pub async fn sync_new_blocks(&mut self) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
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
                let res = clone.update_blocks_to_matured(block_height - 5).await;
                match res {
                    Ok(_) => info!("Blocks till {:?} are matured", block_height - 5),
                    Err(e) => error!("Failed to mature blocks: {:?}", e),
                }
            });
        }

        Ok(())
    }

    pub async fn sync_old_blocks(&mut self) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        Box::pin(async move {
            // Get the latest block from the chain
            let latest_chain = self
                .provider_get_block(BlockNumberOrTag::Latest)
                .await
                .unwrap();

            // Get the latest block from the database
            let mut latest_db = self.storage_get_latest_block_number().await.unwrap();
            if latest_chain.number == latest_db {
                return Ok(());
            }

            // if user specified from which block to start - we need to download this block
            // if we have some block - we need to download next block
            // if we are on the 0 block - we need to download 0 block
            let mut latest_db = match self.sync_mode {
                SyncMode::FromBlock(block) => block,
                SyncMode::FromLastBlockInDB => (latest_db + 1) as i64,
                SyncMode::FromZeroBlock => 0,
            };
            let mut synced = latest_db;

            // Update blocks to matured
            let mut clone = self.clone();
            let update_matured_job: JoinHandle<Result<(), Pin<Box<dyn Error + Send + Sync>>>> =
                spawn(async move {
                    clone
                        .update_blocks_to_matured(latest_chain.number - 5)
                        .await
                });

            info!(
                "Syncing stale blocks from {} to {}",
                latest_db, latest_chain.number
            );

            'outer: loop {
                let mut tasks: Vec<JoinHandle<Result<(), Pin<Box<dyn Error + Send + Sync>>>>> =
                    vec![];
                for _ in 0..10 {
                    let clone = self.clone();
                    tasks.push(spawn(async move {
                        // Get the next block from the chain and add it to the database
                        let (new_block, new_txs) = clone
                            .provider_get_block_with_transactions(BlockNumberOrTag::Number(
                                latest_db as u64,
                            ))
                            .await
                            .unwrap();
                        clone.get_safe_storage().await.add_block(new_block).await?;
                        clone
                            .get_safe_storage()
                            .await
                            .add_transactions(new_txs)
                            .await?;
                        Ok(())
                    }));

                    if latest_chain.number == latest_db {
                        break;
                    }

                    synced += 1;
                    latest_db += 1;

                    if synced % 1000 == 0 {
                        info!("Synced {} blocks", synced);
                    }
                }

                let results = join_all(tasks).await;
                for res in results {
                    match res {
                        Ok(Err(e)) => return Err(e),
                        Ok(_) => (),
                        Err(e) => return Err(Box::pin(e)),
                    }
                }

                if latest_chain.number == latest_db {
                    info!("DB is synced on block {}", latest_chain.number);
                    self.sync_mode = SyncMode::FromLastBlockInDB;
                    self.sync_old_blocks().await?;
                    break 'outer;
                }
            }

            match tokio::join!(update_matured_job).0 {
                Ok(Err(e)) => Err(e),
                Ok(_) => Ok(()),
                Err(e) => Err(Box::pin(e)),
            }
        })
        .await
    }

    pub async fn update_blocks_to_matured(
        &mut self,
        block_height: i64,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        self.storage
            .lock()
            .await
            .update_blocks_to_matured(block_height)
            .await?;
        Ok(())
    }
}

impl ETLWorker {
    async fn storage_get_latest_block_number(
        &self,
    ) -> Result<i64, Pin<Box<dyn Error + Sync + Send>>> {
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
    ) -> Result<(Block, Vec<Transaction>), Pin<Box<dyn Error>>> {
        let res = self.provider.get_block_with_transactions(block).await;
        Ok(res.unwrap())
    }
}
