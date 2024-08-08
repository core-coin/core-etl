#[derive(Debug, Clone)]
pub struct Config {
    /// URL of the RPC node that provides the blockchain data
    pub rpc_url: String,

    /// Block number from which to start the export
    pub block_number: i64,

    /// Whether to continue syncing from the last block in the database
    /// This has precedence over the block_number field
    pub continue_sync: bool,
}
