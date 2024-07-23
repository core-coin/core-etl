use std::error::Error;
use types::Block;

use async_trait::async_trait;

#[async_trait]
pub trait Storage: Send {
    async fn add_block(&mut self, block: Block) -> Result<(), Box<dyn Error>>;

    async fn get_block_by_number(&self, block_number: i64) -> Result<Block, Box<dyn Error>>;
    async fn get_block_by_hash(&self, block_hash: String) -> Result<Block, Box<dyn Error>>;
    async fn get_all_blocks(&self) -> Result<Vec<Block>, Box<dyn Error>>;

    async fn get_latest_block_number(&self) -> Result<i64, Box<dyn Error>>;

    async fn update_blocks_to_matured(&mut self, block_number: i64) -> Result<(), Box<dyn Error>>;

    async fn prepare_db(&mut self) -> Result<(), Box<dyn Error>>;
}
