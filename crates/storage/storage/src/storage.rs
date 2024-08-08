use async_trait::async_trait;
use std::marker::Send;
use std::{error::Error, pin::Pin};
use types::{Block, Transaction};

#[async_trait]
pub trait Storage: Send {
    async fn add_block(&mut self, block: Block) -> Result<(), Pin<Box<dyn Error + Send + Sync>>>;
    async fn add_block_with_replace(
        &mut self,
        block: Block,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>>;

    async fn get_block_by_number(
        &self,
        block_number: i64,
    ) -> Result<Block, Pin<Box<dyn Error + Send + Sync>>>;
    async fn get_block_by_hash(
        &self,
        block_hash: String,
    ) -> Result<Block, Pin<Box<dyn Error + Send + Sync>>>;
    async fn get_all_blocks(&self) -> Result<Vec<Block>, Pin<Box<dyn Error + Send + Sync>>>;
    async fn get_blocks_in_range(
        &self,
        start: i64,
        end: i64,
    ) -> Result<Vec<Block>, Pin<Box<dyn Error + Send + Sync>>>;
    async fn get_latest_block_number(&self) -> Result<i64, Pin<Box<dyn Error + Send + Sync>>>;
    async fn update_blocks_to_matured(
        &mut self,
        block_number: i64,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>>;

    async fn add_transactions(
        &mut self,
        transactions: Vec<Transaction>,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>>;
    async fn get_block_transctions(
        &self,
        block_number: i64,
    ) -> Result<Vec<Transaction>, Pin<Box<dyn Error + Send + Sync>>>;
    async fn get_transaction_by_hash(
        &self,
        hash: String,
    ) -> Result<Transaction, Pin<Box<dyn Error + Send + Sync>>>;

    async fn prepare_db(&mut self) -> Result<(), Pin<Box<dyn Error + Send + Sync>>>;
}
