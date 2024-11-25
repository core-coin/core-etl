use crate::Args;
use mock_storage::MockStorage;
use serde::Serialize;
use sqlite3_storage::Sqlite3Storage;
use std::sync::Arc;
use storage::Storage;
use tracing::info;
use xata_storage::XataStorage;

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StorageType {
    #[default]
    Sqlite3Storage,
    XataStorage,
    MockStorage,
}

impl Args {
    pub async fn choose_storage(&self) -> Arc<dyn Storage + Send + Sync> {
        info!("Storing data about: {:?}", self.modules.clone());
        match self.storage {
            StorageType::MockStorage => Arc::new(MockStorage::new()),
            StorageType::Sqlite3Storage => {
                if self.sqlite3_path.is_none() {
                    panic!("sqlite3_path is required for Sqlite3 Storage");
                }
                let db = Sqlite3Storage::new(
                    self.sqlite3_path.as_ref().unwrap().clone(),
                    self.tables_prefix.clone(),
                    self.modules.clone(),
                )
                .await
                .unwrap();
                db.prepare_db().await.unwrap();
                Arc::new(db)
            }
            StorageType::XataStorage => {
                if self.xata_db_dsn.is_none() {
                    panic!("xata_db_dsn is required for Xata Storage");
                }
                let xata = XataStorage::new(
                    self.xata_db_dsn.as_ref().unwrap().to_string(),
                    self.tables_prefix.clone(),
                    self.modules.clone(),
                )
                .await;
                if xata.is_err() {
                    panic!(
                        "Failed to connect to Xata Storage database: {:?}",
                        xata.err()
                    );
                }
                let xata = xata.unwrap();
                xata.prepare_db().await.unwrap();
                Arc::new(xata)
            }
        }
    }
}
