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

impl From<AtomsBlock> for Block {
    fn from(val: AtomsBlock) -> Self {
        Block {
            difficulty: val.header.difficulty.to_string(),
            energy_limit: val.header.energy_limit as i64,
            energy_used: val.header.energy_used as i64,
            extra_data: val.header.extra_data.encode_hex(),
            hash: val
                .header
                .hash
                .expect("block hash must be set")
                .encode_hex(), // todo:error2215 do smth fix expects
            logs_bloom: val.header.logs_bloom.encode_hex(),
            miner: val.header.miner.to_string(),
            nonce: val
                .header
                .nonce
                .expect("block nonce must be set")
                .encode_hex(), // todo:error2215 do smth fix expects
            number: val.header.number.expect("block number must be set").to_le() as i64, // todo:error2215 do smth fix expects
            parent_hash: val.header.parent_hash.encode_hex(),
            receipts_root: val.header.receipts_root.encode_hex(),
            sha3_uncles: val.header.uncles_hash.encode_hex(),
            state_root: val.header.state_root.encode_hex(),
            timestamp: val.header.timestamp as i64,
            total_difficulty: val
                .header
                .total_difficulty
                .unwrap_or(U256::from(0))
                .to_string(),
            transaction_count: val.transactions.len() as i64,
            transactions_root: val.header.transactions_root.encode_hex(),
            matured: 0,
        }
    }
}
