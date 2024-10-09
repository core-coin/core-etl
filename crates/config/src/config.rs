use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Config {
    /// URL of the RPC node that provides the blockchain data
    pub rpc_url: String,

    /// Block number from which to start the export
    pub block_number: i64,

    /// Watch token transfers. Provide a token type and address to watch
    pub watch_tokens: HashMap<String, HashSet<String>>,

    /// Filter transactions by address
    pub address_filter: Vec<String>,

    /// How long to retain data in the database
    pub retention_duration: i64,

    /// How often to run the cleanup task
    pub cleanup_interval: i64,
}
