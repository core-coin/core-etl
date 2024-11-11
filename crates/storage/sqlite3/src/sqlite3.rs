use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Acquire, Row, Sqlite, SqlitePool};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::format,
    ops::{Deref, DerefMut},
    pin::Pin,
    result,
};
use storage::Storage;
use tokio::time::{self, sleep, Duration};
use tracing::{debug, error, info};
use types::{token_transfer, transaction, Block, TokenTransfer, Transaction, TransferType};

type Result<T> = std::result::Result<T, Pin<Box<dyn Error + Send + Sync>>>;

#[derive(Debug, Clone)]
pub struct Sqlite3Storage {
    pool: SqlitePool,
    tables_prefix: String,
    modules: Vec<String>,
}

impl Sqlite3Storage {
    pub async fn new(db_dsn: String, tables_prefix: String, modules: Vec<String>) -> Result<Self> {
        Self::create_db(db_dsn.clone()).await?;

        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(&db_dsn)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(Self {
            pool,
            tables_prefix,
            modules,
        })
    }

    async fn create_db(db_url: String) -> Result<()> {
        if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            debug!("Creating database {}", db_url);
            Sqlite::create_database(&db_url)
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        } else {
            debug!("Database already exists");
        }
        Ok(())
    }

    pub fn get_db(&self) -> &SqlitePool {
        &self.pool
    }
}

impl Sqlite3Storage {
    /// Migrates the database.
    async fn migrate_db(&self) -> Result<()> {
        debug!("Migrating database");
        let block_hash_foreign_key = if self.modules.contains(&"blocks".to_string()) {
            format!(",
                CONSTRAINT fk_{0}_block_hash FOREIGN KEY (block_hash) REFERENCES {0}_blocks(hash) ON DELETE CASCADE ON UPDATE CASCADE", self.tables_prefix)
        } else {
            "".to_string()
        };
        let queries = vec![
            format!(
                "CREATE TABLE IF NOT EXISTS {}_blocks (
                number INTEGER PRIMARY KEY NOT NULL,
                hash TEXT UNIQUE,
                parent_hash TEXT,
                nonce TEXT,
                sha3_uncles TEXT,
                logs_bloom TEXT,
                transactions_root TEXT,
                state_root TEXT,
                receipts_root TEXT,
                miner TEXT,
                difficulty TEXT,
                total_difficulty TEXT,
                extra_data TEXT,
                energy_limit INTEGER,
                energy_used INTEGER,
                timestamp INTEGER,
                transaction_count INTEGER,
                matured INTEGER,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );",
                self.tables_prefix
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {0}_transactions (
                hash TEXT PRIMARY KEY NOT NULL,
                nonce TEXT,
                block_hash TEXT,
                block_number INTEGER,
                transaction_index INTEGER,
                from_addr TEXT,
                to_addr TEXT,
                value TEXT,
                energy TEXT,
                energy_price TEXT,
                input TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP 
                {1}
            );",
                self.tables_prefix, block_hash_foreign_key
            ),
        ];

        for query in queries {
            sqlx::query(&query)
                .execute(self.get_db())
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        }

        Ok(())
    }
}

#[async_trait]
impl Storage for Sqlite3Storage {
    /// Checks if the database exists. If not, it will be created. Then, the connection to the database will be established and the database will be migrated.
    async fn prepare_db(&self) -> Result<()> {
        self.migrate_db().await?;
        self.create_indexes().await?;
        Ok(())
    }

    async fn create_indexes(&self) -> Result<()> {
        let queries = vec![
            format!("CREATE INDEX IF NOT EXISTS idx_{0}_blocks_hash ON {0}_blocks (hash);", self.tables_prefix),
            format!("CREATE INDEX IF NOT EXISTS idx_{0}_blocks_number ON {0}_blocks (number);", self.tables_prefix),
            format!("CREATE INDEX IF NOT EXISTS idx_{0}_blocks_matured ON {0}_blocks (matured);", self.tables_prefix),
            format!("CREATE INDEX IF NOT EXISTS idx_{0}_transactions_block_hash ON {0}_transactions (block_hash);", self.tables_prefix),
            format!("CREATE INDEX IF NOT EXISTS idx_{0}_transactions_from_addr ON {0}_transactions (from_addr);", self.tables_prefix),
            format!("CREATE INDEX IF NOT EXISTS idx_{0}_transactions_to_addr ON {0}_transactions (to_addr);", self.tables_prefix),
        ];

        for query in queries {
            sqlx::query(&query)
                .execute(self.get_db())
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        }

        Ok(())
    }

    async fn get_latest_block_number(&self) -> Result<i64> {
        let result = sqlx::query_as::<_, Block>(
            format!(
                "SELECT * FROM {}_blocks ORDER BY number DESC LIMIT 1",
                self.tables_prefix
            )
            .as_str(),
        )
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(block) => Ok(block.number),
            Err(sqlx::Error::RowNotFound) => {
                // fetch block number from transactions table
                let result = sqlx::query_as::<_, Transaction>(
                    format!(
                        "SELECT * FROM {}_transactions ORDER BY block_number DESC LIMIT 1",
                        self.tables_prefix
                    )
                    .as_str(),
                )
                .fetch_one(&self.pool)
                .await;
                match result {
                    Ok(tx) => Ok(tx.block_number),
                    Err(sqlx::Error::RowNotFound) => {
                        // fetch block number from any token transfers table
                        let stmt = sqlx::query(
                            format!(
                                "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '{}_%_transfers'", self.tables_prefix).as_str(),
                        ).fetch_all(&self.pool).await.map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
                        let table_name = stmt.first().map(|row| row.get::<String, _>("name"));
                        match table_name {
                            Some(table_name) => {
                                let result = sqlx::query_as::<_, TokenTransfer>(
                                    format!(
                                        "SELECT * FROM {} ORDER BY block_number DESC LIMIT 1",
                                        table_name
                                    )
                                    .as_str(),
                                )
                                .fetch_one(&self.pool)
                                .await;
                                match result {
                                    Ok(tt) => Ok(tt.block_number),
                                    Err(sqlx::Error::RowNotFound) => Ok(0), // No data in the database
                                    Err(e) => Err(Box::pin(e)),
                                }
                            }
                            None => Ok(0), // No data in the database
                        }
                    }
                    Err(e) => Err(Box::pin(e)),
                }
            }
            Err(e) => Err(Box::pin(e)),
        }
    }

    async fn update_blocks_to_matured(&self, block_height: i64) -> Result<()> {
        let result = sqlx::query(
            format!(
                "UPDATE {}_blocks SET matured = 1 WHERE number <= ? AND matured = 0",
                self.tables_prefix
            )
            .as_str(),
        )
        .bind(block_height)
        .execute(self.get_db())
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        debug!("Updated matured blocks: {:?}", result.rows_affected());
        Ok(())
    }

    async fn create_token_transfers_tables(
        &self,
        tokens: HashMap<String, HashSet<String>>,
    ) -> Result<()> {
        for (token, address_set) in tokens {
            for address in address_set {
                let table_name = format!(
                    "{}_{}_{}_transfers",
                    self.tables_prefix,
                    token,
                    &address[..8]
                );
                let tx_hash_foreign_key = if self.modules.contains(&"transactions".to_string()) {
                    format!(", CONSTRAINT fk_{0}_tx_hash FOREIGN KEY (tx_hash) REFERENCES {0}_transactions(hash) ON DELETE CASCADE ON UPDATE CASCADE", self.tables_prefix)
                } else {
                    "".to_string()
                };
                let query = format!(
                    "CREATE TABLE IF NOT EXISTS {table_name} (
                    block_number INTEGER NOT NULL,
                    from_addr TEXT NOT NULL,
                    to_addr TEXT NOT NULL,
                    value TEXT NOT NULL,
                    tx_hash TEXT,
                    address TEXT NOT NULL,
                    transfer_index INTEGER NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    status INTEGER DEFAULT 0
                    {0}
                );",
                    tx_hash_foreign_key
                );
                sqlx::query(&query)
                    .execute(self.get_db())
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
                debug!("Created token transfers table: {}", table_name);
            }
        }
        Ok(())
    }

    async fn clean_block_data(&self, block_number: i64) -> Result<()> {
        let mut tx = self
            .get_db()
            .begin()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        sqlx::query(format!("DELETE FROM {}_blocks WHERE number = ?", self.tables_prefix).as_str())
            .bind(block_number)
            .execute(&mut tx)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        sqlx::query(
            format!(
                "DELETE FROM {}_transactions WHERE block_number = ?",
                self.tables_prefix
            )
            .as_str(),
        )
        .bind(block_number)
        .execute(&mut tx)
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let stmt = sqlx::query(
            format!(
                "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '{}_%_transfers'",
                self.tables_prefix
            )
            .as_str(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let mut table_names: Vec<String> = stmt
            .iter()
            .map(|row| row.get::<String, _>("name"))
            .collect();
        for table in table_names {
            sqlx::query(format!("DELETE FROM {} WHERE block_number = ?", table).as_str())
                .bind(block_number)
                .execute(&mut tx)
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        }

        tx.commit()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        Ok(())
    }

    async fn insert_blocks_with_txs_and_token_transfers(
        &self,
        insert_all: bool,
        blocks: &mut Vec<Block>,
        transactions: &mut Vec<Transaction>,
        token_transfers: &mut HashMap<String, Vec<TokenTransfer>>,
    ) -> Result<()> {
        if blocks.len() > 750 || transactions.len() > 750 || insert_all {
            let mut tx = self
                .get_db()
                .begin()
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
            let mut timestamp_map = HashMap::new();
            let mut block_number_map = HashMap::new();
            if !blocks.is_empty() {
                let query = format!(
                    "INSERT INTO {}_blocks (number, hash, parent_hash, nonce, sha3_uncles, logs_bloom, transactions_root, state_root, receipts_root, miner, difficulty, total_difficulty, extra_data, energy_limit, energy_used, timestamp, transaction_count, matured, created_at) VALUES {}",
                    self.tables_prefix,
                    blocks.iter().map(|block| {
                        let created_at = Utc.timestamp_opt(block.timestamp, 0).unwrap().format("%Y-%m-%d %H:%M:%S").to_string();
                        timestamp_map.insert(&block.hash, created_at.clone()); // Save timestamp for use in transactions
                        format!(
                            "({}, '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', {}, {}, {}, {}, {}, '{}')",
                            block.number, block.hash, block.parent_hash, block.nonce, block.sha3_uncles, block.logs_bloom, block.transactions_root, block.state_root, block.receipts_root, block.miner, block.difficulty, block.total_difficulty, block.extra_data, block.energy_limit, block.energy_used, block.timestamp, block.transaction_count, block.matured, created_at
                        )
                    }).collect::<Vec<_>>().join(", ")
                );
                if self.modules.contains(&"blocks".to_string()) {
                    sqlx::query(&query)
                        .execute(&mut tx)
                        .await
                        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
                    debug!("Inserted blocks: {:?}", blocks.len());
                }
            }

            if !transactions.is_empty() {
                let query = format!(
                    "INSERT INTO {}_transactions (hash, nonce, block_hash, block_number, transaction_index, from_addr, to_addr, value, energy, energy_price, input, created_at) VALUES {}",
                    self.tables_prefix,
                    transactions.iter().map(|tx| {
                        timestamp_map.insert(&tx.hash, timestamp_map.get(&tx.block_hash).unwrap().to_string()); // Save timestamp for use in token transfers
                        block_number_map.insert(&tx.hash, tx.block_number); // save block number so we can set block number in token transfers
                        format!(
                        "('{}', '{}', '{}', {}, {}, '{}', '{}', '{}', '{}', '{}', '{}', '{}')",
                        tx.hash, tx.nonce, tx.block_hash, tx.block_number, tx.transaction_index, tx.from, tx.to, tx.value, tx.energy, tx.energy_price, tx.input, timestamp_map.get(&tx.block_hash).unwrap()
                    )}).collect::<Vec<_>>().join(", ")
                );
                if self.modules.contains(&"transactions".to_string()) {
                    sqlx::query(&query)
                        .execute(&mut tx)
                        .await
                        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
                    debug!("Inserted transactions: {:?}", transactions.len());
                }
            }

            for (table_name, transfers) in token_transfers.clone() {
                if !transfers.is_empty() && self.modules.contains(&"token_transfers".to_string()) {
                    let query = format!(
                        "INSERT INTO {}_{} (block_number, from_addr, to_addr, value, tx_hash, address, transfer_index, created_at, status) VALUES {}",
                        self.tables_prefix,
                        table_name,
                        transfers.iter().map(|tt| format!(
                            "({},'{}', '{}', '{}', '{}', '{}', {}, '{}', {})",
                            block_number_map.get(&tt.tx_hash).unwrap(), tt.from, tt.to, tt.value, tt.tx_hash, tt.address, tt.index, timestamp_map.get(&tt.tx_hash).unwrap(), tt.status
                        )).collect::<Vec<_>>().join(", ")
                    );
                    sqlx::query(&query)
                        .execute(&mut tx)
                        .await
                        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
                    debug!("Inserted token transfers: {:?}", transfers.len());
                }
            }
            debug!("Committing transaction");
            tx.commit()
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
            timestamp_map.clear();
            block_number_map.clear();
            blocks.clear();
            transactions.clear();
            token_transfers.values_mut().for_each(|v| v.clear());
        }
        Ok(())
    }

    async fn start_cleanup_task(&self, interval: Duration, retention_duration: Duration) {
        let pool = self.pool.clone();
        let tables_prefix = self.tables_prefix.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(interval);
            loop {
                interval.tick().await;
                let cutoff =
                    chrono::Utc::now() - chrono::Duration::from_std(retention_duration).unwrap();
                let cutoff_datetime = cutoff.format("%Y-%m-%d %H:%M:%S").to_string();

                let stmt = sqlx::query(format!(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '{}_%_transfers'", tables_prefix).as_str(),
                )
                .fetch_all(&pool)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
                if let Err(e) = stmt {
                    error!(
                        "Failed to delete token transfers table names to clean old records: {:?}",
                        e
                    );
                    continue;
                }

                let mut table_names: Vec<String> = stmt
                    .unwrap()
                    .iter()
                    .map(|row| row.get::<String, _>("name"))
                    .collect();

                table_names.push(format!("{}_blocks", tables_prefix));
                table_names.push(format!("{}_transactions", tables_prefix));

                for table in table_names {
                    let delete_query = format!("DELETE FROM {} WHERE created_at < ?", table);

                    let res = sqlx::query(&delete_query)
                        .bind(&cutoff_datetime)
                        .execute(&pool)
                        .await;
                    if let Err(e) = res {
                        error!("Failed to delete old records from {}: {:?}", table, e);
                    } else {
                        debug!(
                            "Deleted old records from {}. Number of removed rows - {}",
                            table,
                            res.unwrap().rows_affected()
                        );
                    }
                }
            }
        });
    }

    // View functions

    async fn get_token_transfers(
        &self,
        token_address: String,
        from: Option<String>,
        to: Option<String>,
    ) -> Result<Vec<TokenTransfer>> {
        let selector = &token_address[..8];
        let prefix = self.tables_prefix.clone();
        let stmt = sqlx::query(&format!(
            "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '{prefix}_%_{selector}_transfers'"
        ))
        .fetch_all(self.get_db())
        .await .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let table_name: String = stmt.first().map(|row| row.get("name")).unwrap();

        let mut query = format!("SELECT * FROM {table_name} WHERE 1 = 1");
        if let Some(from) = from {
            query += &format!(" AND from_addr = '{}'", from);
        }
        if let Some(to) = to {
            query += &format!(" AND to_addr = '{}'", to);
        }

        let token_transfers = sqlx::query_as::<_, TokenTransfer>(&query)
            .fetch_all(self.get_db())
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(token_transfers)
    }

    async fn get_transaction_token_transfers(&self, tx_hash: String) -> Result<Vec<TokenTransfer>> {
        let stmt = sqlx::query(
            format!(
                "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '{}_%_transfers'",
                self.tables_prefix
            )
            .as_str(),
        )
        .fetch_all(self.get_db())
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let table_names: Vec<String> = stmt.iter().map(|row| row.get("name")).collect();

        let query = table_names
            .iter()
            .map(|table| {
                format!(
                    "SELECT from_addr, to_addr, value, tx_hash, address FROM {} WHERE tx_hash = ?",
                    table
                )
            })
            .collect::<Vec<_>>()
            .join(" UNION ALL ");

        let token_transfers = sqlx::query_as::<_, TokenTransfer>(&query)
            .bind(tx_hash)
            .fetch_all(self.get_db())
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(token_transfers)
    }

    async fn get_address_token_transfers(
        &self,
        address: String,
        transfer_type: TransferType,
    ) -> Result<Vec<TokenTransfer>> {
        let stmt = sqlx::query(
            format!(
                "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '{}_%_transfers'",
                self.tables_prefix
            )
            .as_str(),
        )
        .fetch_all(self.get_db())
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let table_names: Vec<String> = stmt.iter().map(|row| row.get("name")).collect();

        let query = table_names
            .iter()
            .map(|table| {
                let condition = match transfer_type {
                    TransferType::From => format!("from_addr = '{}'", address),
                    TransferType::To => format!("to_addr = '{}'", address),
                    TransferType::All => {
                        format!("from_addr = '{}' OR to_addr = '{}'", address, address)
                    }
                };
                format!("SELECT * FROM {} WHERE {}", table, condition)
            })
            .collect::<Vec<_>>()
            .join(" UNION ALL ");

        let token_transfers = sqlx::query_as::<_, TokenTransfer>(&query)
            .fetch_all(self.get_db())
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(token_transfers)
    }

    async fn get_block_transactions(&self, block_number: i64) -> Result<Vec<Transaction>> {
        let transactions = sqlx::query_as::<_, Transaction>(
            format!(
                "SELECT * FROM {}_transactions WHERE block_number = ?",
                self.tables_prefix
            )
            .as_str(),
        )
        .bind(block_number)
        .fetch_all(self.get_db())
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(transactions)
    }

    async fn get_transaction_by_hash(&self, hash: String) -> Result<Transaction> {
        let transaction = sqlx::query_as::<_, Transaction>(
            format!(
                "SELECT * FROM {}_transactions WHERE hash = ?",
                self.tables_prefix
            )
            .as_str(),
        )
        .bind(hash)
        .fetch_one(self.get_db())
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(transaction)
    }

    async fn get_all_blocks(&self) -> Result<Vec<Block>> {
        let blocks = sqlx::query_as::<_, Block>(
            format!("SELECT * FROM {}_blocks", self.tables_prefix).as_str(),
        )
        .fetch_all(self.get_db())
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(blocks)
    }

    /// Returns a list of blocks in the specified range.
    /// if end is negative, it will return all blocks from start to the latest block.
    async fn get_blocks_in_range(&self, start: i64, end: i64) -> Result<Vec<Block>> {
        let mut query = format!(
            "SELECT * FROM {}_blocks WHERE number >= ? AND number <= ?",
            self.tables_prefix
        );
        if end < 0 {
            query = format!(
                "SELECT * FROM {}_blocks WHERE number >= ?",
                self.tables_prefix
            );
        }
        let blocks = sqlx::query_as::<_, Block>(&query)
            .bind(start)
            .bind(end)
            .fetch_all(self.get_db())
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(blocks)
    }

    async fn get_block_by_number(&self, block_number: i64) -> Result<Block> {
        let block = sqlx::query_as::<_, Block>(
            format!(
                "SELECT * FROM {}_blocks WHERE number = ?",
                self.tables_prefix
            )
            .as_str(),
        )
        .bind(block_number)
        .fetch_one(self.get_db())
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(block)
    }

    async fn get_block_by_hash(&self, block_hash: String) -> Result<Block> {
        let block = sqlx::query_as::<_, Block>(
            format!(
                "SELECT * FROM {}_blocks WHERE hash = '?'",
                self.tables_prefix
            )
            .as_str(),
        )
        .bind(block_hash)
        .fetch_one(self.get_db())
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(block)
    }
}
