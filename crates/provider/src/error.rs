#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("invalid network")]
    InvalidNetwork,
    #[error("invalid block number")]
    InvalidBlockNumber,
    #[error("invalid sync mode")]
    InvalidSyncMode,
    #[error("invalid rpc url")]
    InvalidRpcUrl,
    #[error("invalid watch tokens")]
    InvalidWatchTokens,
    #[error("invalid receipt hash")]
    InvalidReceiptHash,
}
