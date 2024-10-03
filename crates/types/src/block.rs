use atoms_rpc_types::Block as AtomsBlock;
use base_primitives::{hex::ToHexExt, U256};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Clone, Serialize, Deserialize)]
pub struct Block {
    pub number: i64,
    pub hash: String,
    pub parent_hash: String,
    pub nonce: String,
    pub sha3_uncles: String,
    pub logs_bloom: String,
    pub transactions_root: String,
    pub state_root: String,
    pub receipts_root: String,
    pub miner: String,
    pub difficulty: String,
    pub total_difficulty: String,
    pub extra_data: String,
    pub energy_limit: i64,
    pub energy_used: i64,
    pub timestamp: i64,
    pub transaction_count: i64,

    pub matured: i64,
}

impl Into<Block> for AtomsBlock {
    fn into(self) -> Block {
        Block {
            difficulty: self.header.difficulty.to_string(),
            energy_limit: self.header.energy_limit as i64,
            energy_used: self.header.energy_used as i64,
            extra_data: self.header.extra_data.encode_hex(),
            hash: self
                .header
                .hash
                .expect("block hash must be set")
                .encode_hex(), // todo:error2215 do smth fix expects
            logs_bloom: self.header.logs_bloom.encode_hex(),
            miner: self.header.miner.to_string(),
            nonce: self
                .header
                .nonce
                .expect("block nonce must be set")
                .encode_hex(), // todo:error2215 do smth fix expects
            number: self
                .header
                .number
                .expect("block number must be set")
                .to_le() as i64, // todo:error2215 do smth fix expects
            parent_hash: self.header.parent_hash.encode_hex(),
            receipts_root: self.header.receipts_root.encode_hex(),
            sha3_uncles: self.header.uncles_hash.encode_hex(),
            state_root: self.header.state_root.encode_hex(),
            timestamp: self.header.timestamp as i64,
            total_difficulty: self
                .header
                .total_difficulty
                .unwrap_or(U256::from(0))
                .to_string(),
            transaction_count: self.transactions.len() as i64,
            transactions_root: self.header.transactions_root.encode_hex(),
            matured: 0,
        }
    }
}
