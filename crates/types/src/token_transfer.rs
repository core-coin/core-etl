use sqlx::FromRow;

#[derive(Debug, FromRow, Clone)]
pub struct TokenTransfer {
    #[sqlx(rename = "from_addr")]
    pub from: String,
    #[sqlx(rename = "to_addr")]
    pub to: String,
    pub value: String,
    pub tx_hash: String,
    pub address: String,
}
