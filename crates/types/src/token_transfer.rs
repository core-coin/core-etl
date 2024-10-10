use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Clone, Serialize, Deserialize)]
pub struct TokenTransfer {
    pub block_number: i64,
    #[sqlx(rename = "from_addr")]
    pub from: String,
    #[sqlx(rename = "to_addr")]
    pub to: String,
    pub value: String,
    pub tx_hash: String,
    pub address: String,
    #[sqlx(rename = "transfer_index")]
    pub index: i64,
    pub status: i64,
}
