use async_trait::async_trait;
use std::error::Error;
use storage::Storage;
use types::{Block, Transaction};

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
    async fn prepare_db(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn add_block(&mut self, block: Block) -> Result<(), Box<dyn Error>> {
        self.blocks.push(block);
        Ok(())
    }

    async fn get_block_by_number(&self, block_number: i64) -> Result<Block, Box<dyn Error>> {
        for block in &self.blocks {
            if block.number == block_number {
                return Ok(block.clone());
            }
        }
        Err("Block not found".into())
    }

    async fn get_block_by_hash(&self, block_hash: String) -> Result<Block, Box<dyn Error>> {
        for block in &self.blocks {
            if block.hash == block_hash {
                return Ok(block.clone());
            }
        }
        Err("Block not found".into())
    }

    async fn get_all_blocks(&self) -> Result<Vec<Block>, Box<dyn Error>> {
        Ok(self.blocks.clone())
    }

    async fn get_latest_block_number(&self) -> Result<i64, Box<dyn Error>> {
        Ok(self.blocks.last().unwrap().number)
    }

    async fn update_blocks_to_matured(&mut self, _block_number: i64) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn add_transactions(
        &mut self,
        _transactions: Vec<Transaction>,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn get_block_transctions(
        &self,
        _block_number: i64,
    ) -> Result<Vec<Transaction>, Box<dyn Error>> {
        Ok(vec![])
    }

    async fn get_transaction_by_hash(&self, _hash: String) -> Result<Transaction, Box<dyn Error>> {
        Ok(self.transactions.first().unwrap().clone())
    }
}
