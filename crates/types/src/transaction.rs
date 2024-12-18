use atoms_rpc_types::Transaction as AtomsTrasaction;
use base_primitives::{hex::ToHexExt, B256};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub nonce: String,
    pub block_hash: String,
    pub block_number: i64,
    pub transaction_index: i64,
    #[sqlx(rename = "from_addr")]
    pub from: String,
    #[sqlx(rename = "to_addr")]
    pub to: String,
    pub value: String,
    pub energy: String,
    pub energy_price: String,
    pub input: String,
}

impl From<&AtomsTrasaction> for Transaction {
    fn from(val: &AtomsTrasaction) -> Self {
        Transaction {
            block_hash: val.block_hash.unwrap_or(B256::ZERO).encode_hex(),
            block_number: val.block_number.unwrap_or(0) as i64,
            energy: val.energy.to_string(),
            energy_price: val.energy_price.unwrap_or(0).to_string(),
            from: val.from.to_string(),
            hash: val.hash.encode_hex(),
            input: val.input.encode_hex(),
            nonce: val.nonce.to_string(),
            to: { val.to.map(|t| t.to_string()).unwrap_or("".to_string()) },
            transaction_index: { val.transaction_index.map(|t| t as i64).unwrap_or(0) },
            value: val.value.to_string(),
        }
    }
}
