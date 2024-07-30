use crate::Args;
use mock_storage::MockStorage;
use serde::Serialize;
use sqlite3_storage::Sqlite3Storage;
use std::sync::Arc;
use storage::Storage;
use tokio::sync::Mutex;

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StorageType {
    #[default]
    Sqlite3Storage,
    MockStorage,
}

impl Args {
    pub async fn choose_storage(&self) -> Arc<Mutex<dyn Storage>> {
        match self.storage {
            StorageType::MockStorage => Arc::new(Mutex::new(MockStorage::new())),
            StorageType::Sqlite3Storage => {
                let mut db = Sqlite3Storage::new(self.storage_url.clone());
                db.prepare_db().await.unwrap();
                Arc::new(Mutex::new(db))
            }
        }
    }
}
