use crate::ETLError;
use atoms_rpc_types::{BlockNumberOrTag, SyncStatus};
use config::Config;
use contracts::SmartContract;
use futures::future::join_all;
use futures::stream::StreamExt;
use provider::Provider;
use std::collections::HashMap;
use std::pin::Pin;
use std::{error::Error, sync::Arc};
use storage::Storage;
use tokio::spawn;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tracing::info;
use types::{Block, TokenTransfer, Transaction};

pub struct ETLWorker {
    pub config: Config,
    storage: Arc<dyn Storage + Send + Sync>,
    provider: Provider,
    smart_contracts_processors: Vec<Box<dyn SmartContract>>,

    last_saved_block: i64,
    last_checked_block: i64,
}

impl Clone for ETLWorker {
    fn clone(&self) -> Self {
        ETLWorker {
            storage: Arc::clone(&self.storage),
            config: self.config.clone(),
            provider: self.provider.clone(),
            smart_contracts_processors: self.smart_contracts_processors.clone(),
            last_saved_block: self.last_saved_block,
            last_checked_block: self.last_checked_block,
        }
    }
}

impl ETLWorker {
    pub async fn new(
        config: Config,
        storage: Arc<dyn Storage + Send + Sync>,
        provider: Provider,
    ) -> Self {
        let mut etl = ETLWorker {
            config,
            storage,
            provider,
            smart_contracts_processors: vec![],
            last_saved_block: 0,
            last_checked_block: 0,
        };

        if etl.config.watch_tokens.len() > 0 {
            match etl
                .storage
                .create_token_transfers_tables(etl.config.watch_tokens.clone())
                .await
            {
                Ok(_) => {}
                Err(e) => panic!("Failed to create token transfers tables: {:?}", e),
            }

            for (contract_name, address_set) in etl.config.watch_tokens.clone() {
                for contract_address in address_set {
                    etl.smart_contracts_processors
                        .push(etl.select_sc_processor(&contract_name, &contract_address));
                }
            }
        }

        let latest_db_block = etl.storage.get_latest_block_number().await.unwrap_or(0);
        etl.last_saved_block = if latest_db_block != 0 {
            latest_db_block // start from the latest block in the db
        } else {
            etl.config.block_number - 1 // -1 to start from the block_number
        };
        etl
    }

    pub async fn run(&mut self) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        if self.config.retention_duration > 0 {
            let retention_duration = Duration::from_secs(self.config.retention_duration as u64);
            let cleanup_interval = Duration::from_secs(self.config.cleanup_interval as u64);
            self.storage
                .start_cleanup_task(cleanup_interval, retention_duration)
                .await;
        }

        // If lazy mode is enabled, wait until the node is synced
        if self.config.lazy {
            loop {
                let syncing = self.provider.syncing().await.unwrap();
                match syncing {
                    SyncStatus::Info(syncing) => {
                        info!(
                            "Waiting for the node to sync. Current block: {}, highest block: {}",
                            syncing.current_block, syncing.highest_block
                        );
                        sleep(Duration::from_secs(60)).await;
                    }
                    SyncStatus::None => {
                        info!("Node syncing is finished");
                        break;
                    }
                }
            }
        }

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

        while let Some(header) = stream.next().await {
            let block_height = header.header.number.unwrap() as i64;
            let (block, mut transactions, mut token_transfers) =
                self.fetch_and_process_block(block_height).await?;
            info!(
                "Imported new block {:?} with {:?} transactions and {:?} token transfers",
                block.number,
                transactions.len(),
                token_transfers.values().map(|v| v.len()).sum::<usize>()
            );

            if let Err(_) = self
                .safe_insert(
                    true,
                    &mut vec![block.clone()],
                    &mut transactions,
                    &mut token_transfers,
                )
                .await
            {
                info!(
                    "Reorg detected on height {}, cleaning the data and reimporting the block",
                    block.number
                );
                self.storage.clean_block_data(block.number).await?;
                self.safe_insert(
                    true,
                    &mut vec![block],
                    &mut transactions,
                    &mut token_transfers,
                )
                .await?;
            }

            self.update_blocks_to_matured(block_height - 5).await?;
        }

        Ok(())
    }

    pub async fn cleanup_last_blocks(
        &self,
        blocks: i64,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        self.storage.clean_last_blocks(blocks).await?;
        Ok(())
    }

    async fn fetch_and_process_block(
        &self,
        block_number: i64,
    ) -> Result<
        (Block, Vec<Transaction>, HashMap<String, Vec<TokenTransfer>>),
        Pin<Box<dyn Error + Sync + Send>>,
    > {
        let (new_block, mut new_txs) = self
            .provider_get_block_with_transactions(BlockNumberOrTag::Number(block_number as u64))
            .await?;
        let new_token_transfers = self.extract_token_transfers(new_txs.clone()).await?;

        // apply filters
        if !self.config.address_filter.is_empty() {
            new_txs = new_txs
                .into_iter()
                .filter(|tx| {
                    self.config.address_filter.contains(&tx.from)
                        || self.config.address_filter.contains(&tx.to)
                })
                .collect();
        }

        Ok((new_block, new_txs, new_token_transfers))
    }

    async fn process_results(
        &self,
        blocks: &mut Vec<Block>,
        transactions: &mut Vec<Transaction>,
        token_transfers: &mut HashMap<String, Vec<TokenTransfer>>,
        results: Vec<
            Result<
                (Block, Vec<Transaction>, HashMap<String, Vec<TokenTransfer>>),
                Pin<Box<dyn Error + Send + Sync>>,
            >,
        >,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        for res in results {
            match res {
                Ok((block, txs, token_transfers_batch)) => {
                    blocks.push(block);
                    transactions.extend(txs);
                    for (key, values) in token_transfers_batch {
                        token_transfers
                            .entry(key)
                            .or_insert_with(Vec::new)
                            .extend(values);
                    }
                    self.safe_insert(false, blocks, transactions, token_transfers)
                        .await?;
                }
                Err(e) => return Err(Pin::from(e)),
            }
        }
        Ok(())
    }

    async fn safe_insert(
        &self,
        insert_all: bool,
        blocks: &mut Vec<Block>,
        transactions: &mut Vec<Transaction>,
        token_transfers: &mut HashMap<String, Vec<TokenTransfer>>,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        self.storage
            .insert_blocks_with_txs_and_token_transfers(
                insert_all,
                blocks,
                transactions,
                token_transfers,
            )
            .await?;
        Ok(())
    }

    async fn sync_old_blocks(&mut self) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        Box::pin(async move {
            let latest_provider_block = self.provider_get_block(BlockNumberOrTag::Latest).await?;
            self.update_blocks_to_matured(latest_provider_block.number - 5)
                .await?;

            // already synced
            if self.last_saved_block == latest_provider_block.number
            // checked all blocks but the last one do not have data which needs to be stored
                && self.last_checked_block == latest_provider_block.number
            {
                return Ok(());
            }

            let mut block_to_load = self.last_saved_block + 1;

            if block_to_load > latest_provider_block.number {
                return Err(Box::pin(ETLError::ChainIsNotSyncedOnProvider) as _);
            }

            let mut log_counter = block_to_load;

            info!(
                "Syncing stale blocks from {} to {}",
                block_to_load, latest_provider_block.number
            );

            let mut blocks: Vec<Block> = Vec::new();
            let mut transactions: Vec<Transaction> = Vec::new();
            let mut token_transfers: HashMap<String, Vec<TokenTransfer>> = HashMap::new();

            'outer: loop {
                let mut tasks: Vec<
                    JoinHandle<
                        Result<
                            (Block, Vec<Transaction>, HashMap<String, Vec<TokenTransfer>>),
                            Pin<Box<dyn Error + Send + Sync>>,
                        >,
                    >,
                > = vec![];

                for _ in 0..self.config.threads {
                    let clone: ETLWorker = self.clone();
                    let block_number = block_to_load;
                    tasks.push(spawn(async move {
                        clone.fetch_and_process_block(block_number).await
                    }));

                    if latest_provider_block.number == block_to_load {
                        break;
                    }

                    log_counter += 1;
                    block_to_load += 1;

                    if log_counter % 1000 == 0 {
                        info!("Synced {} blocks", log_counter);
                    }
                }

                let results = join_all(tasks)
                    .await
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| Box::from(e))?;
                self.process_results(
                    &mut blocks,
                    &mut transactions,
                    &mut token_transfers,
                    results,
                )
                .await?;

                if latest_provider_block.number == block_to_load {
                    self.safe_insert(true, &mut blocks, &mut transactions, &mut token_transfers)
                        .await?;
                    info!("DB is synced on block {}", latest_provider_block.number);
                    self.last_checked_block = latest_provider_block.number;
                    self.last_saved_block = latest_provider_block.number;
                    self.sync_old_blocks().await?;
                    break 'outer;
                }
            }
            Ok(())
        })
        .await
    }

    pub async fn update_blocks_to_matured(
        &mut self,
        block_height: i64,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        self.storage.update_blocks_to_matured(block_height).await
    }

    async fn extract_token_transfers(
        &self,
        transactions: Vec<Transaction>,
    ) -> Result<HashMap<String, Vec<TokenTransfer>>, Pin<Box<dyn Error + Send + Sync>>> {
        let mut transfers = HashMap::new();
        for tx in transactions {
            for sc in &self.smart_contracts_processors {
                if tx.to != sc.get_address() || !sc.check_if_call(tx.clone().input) {
                    continue;
                }
                let transfer_data = sc.extract_call_data(tx.clone().from, tx.clone().input);
                let receipt = self
                    .provider
                    .get_transaction_receipt(tx.clone().hash)
                    .await?;
                let processor_token_transfers: Vec<TokenTransfer> = transfer_data
                    .into_iter()
                    .map(|(index, from, to, value)| TokenTransfer {
                        block_number: tx.block_number,
                        from,
                        to,
                        value,
                        tx_hash: tx.hash.clone(),
                        address: sc.get_address(),
                        index: index as i64,
                        status: receipt.status().then(|| 1).unwrap_or(0),
                    })
                    .collect();
                if !processor_token_transfers.is_empty() {
                    transfers
                        .entry(sc.get_table_name())
                        .or_insert_with(Vec::new)
                        .extend(processor_token_transfers);
                }
            }
        }

        Ok(transfers)
    }

    async fn provider_get_block(
        &self,
        block: BlockNumberOrTag,
    ) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let block = self.provider.get_block(block).await;
        Ok(block.unwrap())
    }

    async fn provider_get_block_with_transactions(
        &self,
        block: BlockNumberOrTag,
    ) -> Result<(Block, Vec<Transaction>), Pin<Box<dyn Error + Send + Sync>>> {
        let res = self.provider.get_block_with_transactions(block).await;
        Ok(res.unwrap())
    }

    fn select_sc_processor(
        &self,
        contract_name: &str,
        contract_address: &str,
    ) -> Box<dyn SmartContract> {
        match contract_name {
            cbc20::CBC20_NAME => Box::new(cbc20::Cbc20::new(contract_address.to_string())),
            _ => panic!("Unknown contract name"),
        }
    }
}
