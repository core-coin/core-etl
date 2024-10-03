#[derive(Debug, thiserror::Error)]
pub enum ETLError {
    #[error("chain on provider is not synced to requested block yet")]
    ChainIsNotSyncedOnProvider,
}
