use async_trait::async_trait;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::error::Error;
use storage::Storage;
use tracing::{debug, info};
use types::Block;

#[derive(Debug, Clone)]
pub struct Sqlite3Storage {
    pub db_url: String,
    pub db: Option<SqlitePool>,
}

impl Sqlite3Storage {
    pub fn new(db_url: String) -> Self {
        Self { db_url, db: None }
    }

    /// Returns a reference to the database pool.
    ///
    /// # Panics
    ///
    /// This function panics if pool in not prepared and connected.
    fn get_db(&self) -> SqlitePool {
        self.db.clone().unwrap()
    }

    /// Creates the database if it does not exist.
    async fn create_db(&self) -> Result<(), Box<dyn Error>> {
        if !Sqlite::database_exists(&self.db_url).await.unwrap_or(false) {
            debug!("Creating database {}", &self.db_url);
            Sqlite::create_database(&self.db_url).await?;
        } else {
            debug!("Database already exists");
        }
        Ok(())
    }
    /// Connects to the database.
    async fn connect_to_db(&mut self) -> Result<(), Box<dyn Error>> {
        self.db = Some(SqlitePool::connect(&self.db_url.clone()).await.unwrap());
        info!("Connected to database at path {}", &self.db_url);

        Ok(())
    }
    /// Migrates the database.
    async fn migrate_db(&self) -> Result<(), Box<dyn Error>> {
        debug!("Migrating database");
        let result = sqlx::query(
            "CREATE TABLE IF NOT EXISTS blocks (
        number INTEGER PRIMARY KEY NOT NULL,
        hash TEXT NOT NULL,
        parent_hash TEXT NOT NULL,
        nonce INTEGER NOT NULL,
        sha3_uncles TEXT NOT NULL,
        logs_bloom TEXT NOT NULL,
        transactions_root TEXT NOT NULL,
        state_root TEXT NOT NULL,
        receipts_root TEXT NOT NULL,
        miner TEXT NOT NULL,
        difficulty TEXT NOT NULL,
        total_difficulty TEXT NOT NULL,
        extra_data TEXT NOT NULL,
        energy_limit INTEGER NOT NULL,
        energy_used INTEGER NOT NULL,
        timestamp INTEGER NOT NULL,
        transaction_count INTEGER NOT NULL)
        ;",
        )
        .execute(&self.get_db())
        .await
        .unwrap();
        debug!("Create blocks table result: {:?}", result);
        Ok(())
    }
}

#[async_trait]
impl Storage for Sqlite3Storage {
    /// Checks if the database exists. If not, it will be created. Then, the connection to the database will be established and the database will be migrated.
    async fn prepare_db(&mut self) -> Result<(), Box<dyn Error>> {
        self.create_db().await?;
        self.connect_to_db().await?;
        self.migrate_db().await?;
        Ok(())
    }

    async fn add_block(&mut self, block: Block) -> Result<(), Box<dyn Error>> {
        let result = sqlx::query("INSERT INTO blocks (number, hash, parent_hash, nonce, sha3_uncles, logs_bloom, transactions_root, state_root, receipts_root, miner, difficulty, total_difficulty, extra_data, energy_limit, energy_used, timestamp, transaction_count) VALUES (?, ? , ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(block.number)
            .bind(block.hash)
            .bind(block.parent_hash)
            .bind(block.nonce)
            .bind(block.sha3_uncles)
            .bind(block.logs_bloom)
            .bind(block.transactions_root)
            .bind(block.state_root)
            .bind(block.receipts_root)
            .bind(block.miner)
            .bind(block.difficulty)
            .bind(block.total_difficulty)
            .bind(block.extra_data)
            .bind(block.energy_limit)
            .bind(block.energy_used)
            .bind(block.timestamp)
            .bind(block.transaction_count)

            .execute(&self.get_db())
            .await?;
        debug!("Added block to db: {:?}", block.number);
        if result.rows_affected() == 1 {
            Ok(())
        } else {
            Err("Failed to add block".into())
        }
    }

    async fn get_all_blocks(&self) -> Result<Vec<Block>, Box<dyn Error>> {
        let blocks = sqlx::query_as::<_, Block>("SELECT * FROM blocks")
            .fetch_all(&self.get_db())
            .await?;
        Ok(blocks)
    }

    async fn get_block_by_number(&self, block_number: i64) -> Result<Block, Box<dyn Error>> {
        let block = sqlx::query_as::<_, Block>("SELECT * FROM blocks WHERE number = ?")
            .bind(block_number)
            .fetch_one(&self.get_db())
            .await?;
        Ok(block)
    }

    async fn get_block_by_hash(&self, block_hash: String) -> Result<Block, Box<dyn Error>> {
        let block = sqlx::query_as::<_, Block>("SELECT * FROM blocks WHERE hash = '?'")
            .bind(block_hash)
            .fetch_one(&self.get_db())
            .await?;
        Ok(block)
    }

    async fn get_latest_block_number(&self) -> Result<i64, Box<dyn Error>> {
        let result =
            sqlx::query_as::<_, Block>("SELECT * FROM blocks ORDER BY number DESC LIMIT 1")
                .fetch_one(&self.get_db())
                .await;
        match result {
            Ok(block) => Ok(block.number),
            Err(sqlx::Error::RowNotFound) => Ok(0),
            Err(e) => Err(Box::new(e)),
        }
    }

    async fn update_blocks_to_matured(&mut self, block_height: i64) -> Result<(), Box<dyn Error>> {
        let result = sqlx::query("UPDATE blocks SET matured = 1 WHERE number <= ?")
            .bind(block_height)
            .execute(&self.get_db())
            .await?;
        debug!("Updated matured blocks: {:?}", result.rows_affected());
        Ok(())
    }
}
