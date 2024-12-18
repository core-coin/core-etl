use crate::Args;
use mock_storage::MockStorage;
use postgres_storage::PostgresStorage;
use serde::Serialize;
use sqlite3_storage::Sqlite3Storage;
use std::sync::Arc;
use storage::Storage;
use tracing::info;

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StorageType {
    #[default]
    Sqlite3,
    Postgres,
    Mock,
}

impl Args {
    pub async fn choose_storage(&self) -> Arc<dyn Storage + Send + Sync> {
        info!("Storing data about: {:?}", self.modules.clone());
        match self.storage {
            StorageType::Mock => Arc::new(MockStorage::new()),
            StorageType::Sqlite3 => {
                if self.sqlite3_path.is_none() {
                    panic!("sqlite3_path is required for Sqlite3 Storage");
                }
                info!(
                    "Connecting to Sqlite3 Storage database: {:?}",
                    self.sqlite3_path
                );
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
            StorageType::Postgres => {
                if self.postgres_db_dsn.is_none() {
                    panic!("postgres_db_dsn is required for Postgres Storage");
                }
                info!(
                    "Connecting to Postgres Storage database: {:?}",
                    self.postgres_db_dsn
                );
                let postgres = PostgresStorage::new(
                    self.postgres_db_dsn.as_ref().unwrap().to_string(),
                    self.tables_prefix.clone(),
                    self.modules.clone(),
                )
                .await;
                if postgres.is_err() {
                    panic!(
                        "Failed to connect to Postgres Storage database: {:?}",
                        postgres.err()
                    );
                }
                let postgres = postgres.unwrap();
                postgres.prepare_db().await.unwrap();
                Arc::new(postgres)
            }
        }
    }
}
