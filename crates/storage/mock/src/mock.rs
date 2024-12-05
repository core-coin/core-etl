use async_trait::async_trait;
use std::{collections::HashSet, error::Error, fmt::Error as fmt_err, pin::Pin};
use storage::Storage;
use types::{Block, TokenTransfer, Transaction, TransferType};

#[derive(Debug, Clone)]
pub struct MockStorage {
    pub blocks: Vec<Block>,
    pub transactions: Vec<Transaction>,
}

impl MockStorage {
    pub fn new() -> Self {
        Self {
            blocks: vec![],
            transactions: vec![],
        }
    }
}

#[async_trait]
impl Storage for MockStorage {
    async fn prepare_db(&self) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        Ok(())
    }

    async fn get_block_by_number(
        &self,
        block_number: i64,
    ) -> Result<Block, Pin<Box<dyn Error + Send + Sync>>> {
        for block in &self.blocks {
            if block.number == block_number {
                return Ok(block.clone());
            }
        }
        Err(Box::pin(fmt_err))
    }

    async fn get_block_by_hash(
        &self,
        block_hash: String,
    ) -> Result<Block, Pin<Box<dyn Error + Send + Sync>>> {
        for block in &self.blocks {
            if block.hash == block_hash {
                return Ok(block.clone());
            }
        }
        Err(Box::pin(fmt_err))
    }

    async fn get_all_blocks(&self) -> Result<Vec<Block>, Pin<Box<dyn Error + Send + Sync>>> {
        Ok(self.blocks.clone())
    }

    async fn get_blocks_in_range(
        &self,
        start: i64,
        end: i64,
    ) -> Result<Vec<Block>, Pin<Box<dyn Error + Send + Sync>>> {
        let mut blocks = vec![];
        for block in &self.blocks {
            if block.number >= start && block.number <= end {
                blocks.push(block.clone());
            }
        }
        Ok(blocks)
    }

    async fn get_latest_block_number(&self) -> Result<i64, Pin<Box<dyn Error + Send + Sync>>> {
        Ok(self.blocks.last().unwrap().number)
    }

    async fn update_blocks_to_matured(
        &self,
        _block_number: i64,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        Ok(())
    }

    async fn get_block_transactions(
        &self,
        _block_number: i64,
    ) -> Result<Vec<Transaction>, Pin<Box<dyn Error + Send + Sync>>> {
        Ok(vec![])
    }

    async fn get_transaction_by_hash(
        &self,
        _hash: String,
    ) -> Result<Transaction, Pin<Box<dyn Error + Send + Sync>>> {
        Ok(self.transactions.first().unwrap().clone())
    }

    async fn create_token_transfers_tables(
        &self,
        _tokens: std::collections::HashMap<String, HashSet<String>>,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        Ok(())
    }

    async fn get_token_transfers(
        &self,
        _token_address: String,
        _from: Option<String>,
        _to: Option<String>,
    ) -> Result<Vec<TokenTransfer>, Pin<Box<dyn Error + Send + Sync>>> {
        Ok(vec![])
    }

    async fn get_transaction_token_transfers(
        &self,
        _tx_hash: String,
    ) -> Result<Vec<TokenTransfer>, Pin<Box<dyn Error + Send + Sync>>> {
        Ok(vec![])
    }

    async fn get_address_token_transfers(
        &self,
        _address: String,
        _transfer_type: TransferType,
    ) -> Result<Vec<TokenTransfer>, Pin<Box<dyn Error + Send + Sync>>> {
        Ok(vec![])
    }

    // async fn add_blocks(
    //     &mut self,
    //     blocks: Vec<Block>,
    // ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
    //     self.blocks.extend(blocks);
    //     Ok(())
    // }

    async fn create_indexes(&self) -> Result<(), Pin<Box<dyn Error + Send + Sync>>>
    where
        Self: Sized,
    {
        Ok(())
    }

    async fn clean_block_data(
        &self,
        _block_number: i64,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>>
    where
        Self: Sized,
    {
        Ok(())
    }

    async fn clean_last_blocks(&self, _number: i64) -> Result<(), Pin<Box<dyn Error + Send + Sync>>>
    where
        Self: Sized,
    {
        Ok(())
    }

    async fn insert_blocks_with_txs_and_token_transfers(
        &self,
        _insert_all: bool,
        _blocks: &mut Vec<Block>,
        _transactions: &mut Vec<Transaction>,
        _token_transfers: &mut std::collections::HashMap<String, Vec<TokenTransfer>>,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        Ok(())
    }

    async fn start_cleanup_task(
        &self,
        _interval: tokio::time::Duration,
        _retention_duration: tokio::time::Duration,
    ) {
    }
}
