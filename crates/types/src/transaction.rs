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

impl Into<Transaction> for &AtomsTrasaction {
    fn into(self) -> Transaction {
        Transaction {
            block_hash: self.block_hash.unwrap_or(B256::ZERO).encode_hex(),
            block_number: self.block_number.unwrap_or(0) as i64,
            energy: self.energy.to_string(),
            energy_price: self.energy_price.unwrap_or(0).to_string(),
            from: self.from.to_string(),
            hash: self.hash.encode_hex(),
            input: self.input.encode_hex(),
            // network_id: self.network_id as i64,
            nonce: self.nonce.to_string(),
            // signature: {
            //     self.signature
            //         .map(|t| t.sig().encode_hex())
            //         .unwrap_or("".to_string())
            // },
            to: { self.to.map(|t| t.to_string()).unwrap_or("".to_string()) },
            transaction_index: { self.transaction_index.map(|t| t as i64).unwrap_or(0) },
            value: self.value.to_string(),
        }
    }
}
