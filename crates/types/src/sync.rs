/// SyncMode is an enum that represents the different types of sync that can be done
#[derive(Debug, Clone)]
pub enum SyncMode {
    // Sync from the zero block
    FromZeroBlock,
    // Sync from a specific block
    FromBlock(i64),
    // Sync from the last block in the database
    FromLastBlockInDB,
}
