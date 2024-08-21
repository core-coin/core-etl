use async_trait::async_trait;
use std::{error::Error, fmt::Error as fmt_err, pin::Pin};
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
    async fn prepare_db(&mut self) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        Ok(())
    }

    async fn add_block(&mut self, block: Block) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        self.blocks.push(block);
        Ok(())
    }

    async fn add_block_with_replace(
        &mut self,
        block: Block,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        self.blocks.push(block);
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
        &mut self,
        _block_number: i64,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        Ok(())
    }

    async fn add_transactions(
        &mut self,
        _transactions: Vec<Transaction>,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        Ok(())
    }

    async fn get_block_transctions(
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
}
