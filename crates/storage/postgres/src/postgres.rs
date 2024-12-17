use std::{
    collections::{HashMap, HashSet},
    error::Error,
    pin::Pin,
};

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use storage::Storage;
use tokio::time::{self, Duration};
use tracing::{debug, error};
use types::{Block, TokenTransfer, Transaction, TransferType};

use crate::error::PostgresStorageError;

#[derive(Debug, Clone)]
pub struct PostgresStorage {
    pub db_dsn: String,
    pub pool: PgPool,
    pub tables_prefix: String,
    pub modules: Vec<String>,
}

impl PostgresStorage {
    pub async fn new(
        db_dsn: String,
        tables_prefix: String,
        modules: Vec<String>,
    ) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10) // Adjust the number of connections as needed
            .acquire_timeout(Duration::from_secs(60))
            .connect(&db_dsn)
            .await?;

        Ok(Self {
            db_dsn,
            pool,
            tables_prefix,
            modules,
        })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        debug!("Migrating database tables");
        let block_hash_foreign_key = if self.modules.contains(&"blocks".to_string()) {
            format!(
                "REFERENCES {0}_blocks(hash) ON DELETE CASCADE ON UPDATE CASCADE",
                self.tables_prefix
            )
        } else {
            "".to_string()
        };
        let create_blocks_table = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {}_blocks (
                number BIGINT PRIMARY KEY,
                hash VARCHAR(64) UNIQUE,
                parent_hash VARCHAR(64),
                nonce VARCHAR(64),
                sha3_uncles VARCHAR(64),
                logs_bloom TEXT,
                transactions_root VARCHAR(64),
                state_root VARCHAR(64),
                receipts_root VARCHAR(64),
                miner VARCHAR(44),
                difficulty VARCHAR(64),
                total_difficulty VARCHAR(64),
                extra_data TEXT,
                energy_limit BIGINT,
                energy_used BIGINT,
                timestamp BIGINT,
                transaction_count BIGINT,
                matured BIGINT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
        "#,
            self.tables_prefix
        );

        let create_transactions_table = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {0}_transactions (
                hash VARCHAR(64) PRIMARY KEY,
                nonce VARCHAR(64),
                block_hash VARCHAR(64) {1},
                block_number BIGINT,
                transaction_index BIGINT,
                from_addr VARCHAR(44),
                to_addr VARCHAR(44),
                value VARCHAR(64),
                energy VARCHAR(64),
                energy_price VARCHAR(64),
                input TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
        "#,
            self.tables_prefix, block_hash_foreign_key
        );

        sqlx::query(&create_blocks_table)
            .execute(&self.pool)
            .await?;
        sqlx::query(&create_transactions_table)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl Storage for PostgresStorage {
    async fn prepare_db(&self) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        self.migrate().await.map_err(PostgresStorageError::from)?;
        self.create_indexes().await?;
        Ok(())
    }

    async fn create_indexes(&self) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        let indexes = vec![
            format!("CREATE INDEX IF NOT EXISTS idx_{0}_blocks_hash ON {0}_blocks (hash);", self.tables_prefix),
            format!("CREATE INDEX IF NOT EXISTS idx_{0}_blocks_number ON {0}_blocks (number);", self.tables_prefix),
            format!("CREATE INDEX IF NOT EXISTS idx_{0}_blocks_matured ON {0}_blocks (matured);", self.tables_prefix),
            format!("CREATE INDEX IF NOT EXISTS idx_{0}_transactions_block_hash ON {0}_transactions(block_hash);", self.tables_prefix),
            format!("CREATE INDEX IF NOT EXISTS idx_{0}_transactions_from_addr ON {0}_transactions(from_addr);", self.tables_prefix),
            format!("CREATE INDEX IF NOT EXISTS idx_{0}_transactions_to_addr ON {0}_transactions(to_addr);", self.tables_prefix),
        ];

        for index in indexes {
            sqlx::query(&index)
                .execute(&self.pool)
                .await
                .map_err(PostgresStorageError::from)?;
        }
        Ok(())
    }

    async fn get_latest_block_number(&self) -> Result<i64, Pin<Box<dyn Error + Send + Sync>>> {
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
                                "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_name LIKE '{}_%_transfers';",
                                self.tables_prefix
                            )
                            .as_str(),
                        ).fetch_all(&self.pool).await.map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
                        let table_name = stmt.first().map(|row| row.get::<String, _>("table_name"));
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

    async fn update_blocks_to_matured(
        &self,
        from: i64,
        to: i64,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        let result = sqlx::query(
            format!(
                "UPDATE {}_blocks SET matured = 1 WHERE number >= {} AND number <= {} AND matured = 0",
                self.tables_prefix, from, to
            )
            .as_str(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        debug!("Updated matured blocks: {:?}", result.rows_affected());
        Ok(())
    }

    async fn create_token_transfers_tables(
        &self,
        tokens: HashMap<String, HashSet<String>>,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        for (token, address_set) in tokens {
            for address in address_set {
                let table_name = format!(
                    "{}_{}_{}_transfers",
                    self.tables_prefix,
                    token,
                    &address[..8]
                );
                let tx_hash_foreign_key = if self.modules.contains(&"transactions".to_string()) {
                    format!(
                        "REFERENCES {0}_transactions(hash) ON DELETE CASCADE ON UPDATE CASCADE",
                        self.tables_prefix
                    )
                } else {
                    "".to_string()
                };
                let create_table_query = format!(
                    "CREATE TABLE IF NOT EXISTS {table_name} (
                    id SERIAL PRIMARY KEY,
                    block_number BIGINT,
                    from_addr VARCHAR(44) NOT NULL,
                    to_addr VARCHAR(44) NOT NULL,
                    value VARCHAR(64) NOT NULL,
                    tx_hash VARCHAR(64) {0},
                    address VARCHAR(44) NOT NULL,
                    transfer_index BIGINT NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    status BIGINT DEFAULT 0,
                    UNIQUE (tx_hash, transfer_index)
                );",
                    tx_hash_foreign_key
                );

                let result = sqlx::query(&create_table_query)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

                debug!(
                    "Create {:?} token transfers table result: {:?}",
                    table_name, result
                );
            }
        }
        Ok(())
    }

    async fn clean_block_data(
        &self,
        block_number: i64,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let delete_blocks_query = format!(
            "DELETE FROM {}_blocks WHERE number = {}",
            self.tables_prefix, block_number
        );
        sqlx::query(&delete_blocks_query)
            .execute(&mut tx)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let delete_txs_query = format!(
            "DELETE FROM {}_transactions WHERE block_number = {}",
            self.tables_prefix, block_number
        );
        sqlx::query(&delete_txs_query)
            .execute(&mut tx)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let stmt = sqlx::query(
                format!("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_name LIKE '{}_%_transfers';", self.tables_prefix).as_str(),
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let table_names: Vec<String> = stmt
            .iter()
            .map(|row| row.get::<String, _>("table_name"))
            .collect();
        for table in table_names {
            let delete_transfers_query = format!(
                "DELETE FROM {} WHERE block_number = {}",
                table, block_number
            );
            sqlx::query(&delete_transfers_query)
                .execute(&mut tx)
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        }

        tx.commit()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        Ok(())
    }

    async fn clean_last_blocks(
        &self,
        number: i64,
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let delete_blocks_query = format!(
            "DELETE FROM {}_blocks WHERE number > (SELECT max(number) FROM {}_blocks) - {}",
            self.tables_prefix, self.tables_prefix, number
        );
        sqlx::query(&delete_blocks_query)
            .execute(&mut tx)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let delete_txs_query = format!(
            "DELETE FROM {}_transactions WHERE block_number > (SELECT max(block_number) FROM {}_transactions) - {}",
            self.tables_prefix, self.tables_prefix, number
        );
        sqlx::query(&delete_txs_query)
            .execute(&mut tx)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let stmt = sqlx::query(
                format!("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_name LIKE '{}_%_transfers';", self.tables_prefix).as_str(),
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let table_names: Vec<String> = stmt
            .iter()
            .map(|row| row.get::<String, _>("table_name"))
            .collect();
        for table in table_names {
            let delete_transfers_query = format!(
                "DELETE FROM {} WHERE block_number > (SELECT max(block_number) FROM {}) - {}",
                table, table, number
            );
            sqlx::query(&delete_transfers_query)
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
    ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        if blocks.len() > 500 || transactions.len() > 500 || insert_all {
            let mut tx = self
                .pool
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
                    )}).collect::<Vec<_>>().join(", ")
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
                    transactions.iter().map(|tx|{
                        timestamp_map.insert(&tx.hash, timestamp_map.get(&tx.block_hash).unwrap().to_string()); // Save timestamp for use in token transfers
                        block_number_map.insert(&tx.hash, tx.block_number); // Save block number for use in token transfers
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
                            "({}, '{}', '{}', '{}', '{}', '{}', {}, '{}', {})",
                            block_number_map.get(&tt.tx_hash).unwrap(),  tt.from, tt.to, tt.value, tt.tx_hash, tt.address, tt.index, timestamp_map.get(&tt.tx_hash).unwrap(), tt.status
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
                let cutoff_timestamp = cutoff.timestamp();

                let stmt = sqlx::query(format!(
                    "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_name LIKE '{}_%_transfers';", tables_prefix).as_str(),
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
                    .map(|row| row.get::<String, _>("table_name"))
                    .collect();

                table_names.push(format!("{}_blocks", tables_prefix));
                table_names.push(format!("{}_transactions", tables_prefix));

                for table in table_names {
                    let delete_query = format!(
                        "DELETE FROM {} WHERE created_at < to_timestamp({})",
                        table, cutoff_timestamp
                    );

                    let res = sqlx::query(&delete_query).execute(&pool).await;
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
    ) -> Result<Vec<TokenTransfer>, Pin<Box<dyn Error + Send + Sync>>> {
        let selector = &token_address[..8];
        let prefix = self.tables_prefix.clone();
        let stmt = sqlx::query(
            format!(
                "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_name LIKE '{prefix}_%_{selector}_transfers'"
            )
            .as_str(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let table_name: String = stmt
            .first()
            .map(|row| row.get::<String, _>("table_name"))
            .unwrap();

        let mut query = format!("SELECT * FROM {table_name} WHERE 1 = 1");
        if let Some(from) = from {
            query += &format!(" AND from_addr = '{}'", from);
        }
        if let Some(to) = to {
            query += &format!(" AND to_addr = '{}'", to);
        }

        let token_transfers = sqlx::query_as::<_, TokenTransfer>(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        Ok(token_transfers)
    }

    async fn get_transaction_token_transfers(
        &self,
        tx_hash: String,
    ) -> Result<Vec<TokenTransfer>, Pin<Box<dyn Error + Send + Sync>>> {
        let stmt = sqlx::query(
            format!("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_name LIKE '{}_%_transfers';", self.tables_prefix).as_str(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let table_names: Vec<String> = stmt
            .iter()
            .map(|row| row.get::<String, _>("table_name"))
            .collect();

        let mut query_parts = Vec::new();
        for table in &table_names {
            query_parts.push(format!(
                "SELECT from_addr, to_addr, value, tx_hash, address FROM {} WHERE tx_hash = '{}'",
                table, tx_hash
            ));
        }
        let query = query_parts.join(" UNION ALL ");

        let token_transfers = sqlx::query_as::<_, TokenTransfer>(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(token_transfers)
    }

    async fn get_address_token_transfers(
        &self,
        address: String,
        transfer_type: TransferType,
    ) -> Result<Vec<TokenTransfer>, Pin<Box<dyn Error + Send + Sync>>> {
        let stmt = sqlx::query( format!(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_name LIKE '{}_%_transfers';", self.tables_prefix).as_str(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let table_names: Vec<String> = stmt
            .iter()
            .map(|row| row.get::<String, _>("table_name"))
            .collect();

        let mut query_parts = Vec::new();
        for table in &table_names {
            let mut subquery = format!("SELECT * FROM {} WHERE ", table);
            match transfer_type {
                TransferType::From => subquery += &format!("from_addr = '{}'", address),
                TransferType::To => subquery += &format!("to_addr = '{}'", address),
                TransferType::All => {
                    subquery += &format!("from_addr = '{}' OR to_addr = '{}'", address, address)
                }
            }
            query_parts.push(subquery);
        }
        let query = query_parts.join(" UNION ALL ");

        let token_transfers = sqlx::query_as::<_, TokenTransfer>(&query)
            .bind(address)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(token_transfers)
    }

    async fn get_block_transactions(
        &self,
        block_number: i64,
    ) -> Result<Vec<Transaction>, Pin<Box<dyn Error + Send + Sync>>> {
        let transactions = sqlx::query_as::<_, Transaction>(
            format!(
                "SELECT * FROM {}_transactions WHERE block_number = {}",
                self.tables_prefix, block_number
            )
            .as_str(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        Ok(transactions)
    }

    async fn get_transaction_by_hash(
        &self,
        hash: String,
    ) -> Result<Transaction, Pin<Box<dyn Error + Send + Sync>>> {
        let transaction = sqlx::query_as::<_, Transaction>(
            format!(
                "SELECT * FROM {}_transactions WHERE hash = '{}'",
                self.tables_prefix, hash
            )
            .as_str(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        Ok(transaction)
    }

    async fn get_all_blocks(&self) -> Result<Vec<Block>, Pin<Box<dyn Error + Send + Sync>>> {
        let blocks = sqlx::query_as::<_, Block>(
            format!("SELECT * FROM {}_blocks", self.tables_prefix).as_str(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(blocks)
    }

    async fn get_blocks_in_range(
        &self,
        start: i64,
        end: i64,
    ) -> Result<Vec<Block>, Pin<Box<dyn Error + Send + Sync>>> {
        let query = if end < 0 {
            format!(
                "SELECT * FROM {}_blocks WHERE number >= {}",
                self.tables_prefix, start
            )
        } else {
            format!(
                "SELECT * FROM {}_blocks WHERE number >= {} AND number <= {}",
                self.tables_prefix, start, end
            )
        };

        let blocks = sqlx::query_as::<_, Block>(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(blocks)
    }

    async fn get_block_by_number(
        &self,
        block_number: i64,
    ) -> Result<Block, Pin<Box<dyn Error + Send + Sync>>> {
        let block = sqlx::query_as::<_, Block>(
            format!(
                "SELECT * FROM {}_blocks WHERE number = {}",
                self.tables_prefix, block_number
            )
            .as_str(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(block)
    }

    async fn get_block_by_hash(
        &self,
        block_hash: String,
    ) -> Result<Block, Pin<Box<dyn Error + Send + Sync>>> {
        let block = sqlx::query_as::<_, Block>(
            format!(
                "SELECT * FROM {}_blocks WHERE hash = '{}'",
                self.tables_prefix, block_hash
            )
            .as_str(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(block)
    }
}
