use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::marker::Send;
use std::{error::Error, pin::Pin};
use tokio::time::Duration;
use types::{Block, TokenTransfer, Transaction, TransferType};

#[async_trait]
pub trait Storage: Send + Sync {
    /// Prepare the database for use. This should be called before any other methods.
    /// This should create the necessary tables.
    async fn prepare_db(&self) -> Result<(), Pin<Box<dyn Error + Send + Sync>>>;
    /// Create indexes
    async fn create_indexes(&self) -> Result<(), Pin<Box<dyn Error + Send + Sync>>>;
    /// Get the latest block number in the database
    async fn get_latest_block_number(&self) -> Result<i64, Pin<Box<dyn Error + Send + Sync>>>;
    /// Update blocks to matured
    async fn update_blocks_to_matured(
        &self,
        block_number: i64,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>>;
    /// Create all token transfers tables
    async fn create_token_transfers_tables(
        &self,
        tokens: HashMap<String, HashSet<String>>,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>>;
    /// Clean block data with all related transactions and token transfers
    async fn clean_block_data(
        &self,
        block_number: i64,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>>;
    /// Insert blocks with transactions and token transfers
    async fn insert_blocks_with_txs_and_token_transfers(
        &self,
        insert_all: bool,
        blocks: &mut Vec<Block>,
        transactions: &mut Vec<Transaction>,
        token_transfers: &mut HashMap<String, Vec<TokenTransfer>>,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>>;

    async fn start_cleanup_task(&self, interval: Duration, retention_duration: Duration);

    // View functions
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

    async fn get_block_transactions(
        &self,
        block_number: i64,
    ) -> Result<Vec<Transaction>, Pin<Box<dyn Error + Send + Sync>>>;
    async fn get_transaction_by_hash(
        &self,
        hash: String,
    ) -> Result<Transaction, Pin<Box<dyn Error + Send + Sync>>>;
    async fn get_token_transfers(
        &self,
        token_address: String,
        from: Option<String>,
        to: Option<String>,
    ) -> Result<Vec<TokenTransfer>, Pin<Box<dyn Error + Send + Sync>>>;

    async fn get_transaction_token_transfers(
        &self,
        tx_hash: String,
    ) -> Result<Vec<TokenTransfer>, Pin<Box<dyn Error + Send + Sync>>>;

    async fn get_address_token_transfers(
        &self,
        address: String,
        transfer_type: TransferType,
    ) -> Result<Vec<TokenTransfer>, Pin<Box<dyn Error + Send + Sync>>>;
}
