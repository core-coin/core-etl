// use std::{
//     collections::HashMap,
//     env,
//     error::Error,
//     hash::Hash,
//     ops::{Deref, DerefMut},
//     pin::Pin,
//     process::Command,
//     result,
// };

// use async_trait::async_trait;
// use serde::Deserialize;
// use serde_json::json;
// use sqlx::Execute;
// use storage::Storage;
// use tracing::debug;
// use types::{token_transfer, transaction, Block, TokenTransfer, Transaction, TransferType};
// #[derive(Clone)]
// pub struct D1Storage {
//     db_name: String,
//     api_token: String,
// }

// #[derive(Debug, Deserialize, Clone)]
// #[serde(untagged)]
// pub enum D1APIResult {
//     Block(Block),
//     Transaction(Transaction),
//     TokenTransfer(TokenTransfer),
//     SQLTable { name: String },
// }

// #[derive(Debug, Deserialize, Clone)]
// pub struct D1APIError {
//     pub text: String,
//     pub notes: Vec<D1APINote>,
//     pub kind: String,
//     pub name: String,
//     pub code: u32,
//     pub account_tag: String,
// }

// #[derive(Debug, Deserialize, Clone)]
// pub struct D1APINote {
//     pub text: String,
// }

// #[derive(Debug, Deserialize, Clone)]
// pub struct D1APIMeta {
//     pub served_by: String,
//     pub duration: f64,
//     pub changes: u32,
//     pub last_row_id: Option<u32>,
//     pub changed_db: bool,
//     pub size_after: u32,
//     pub rows_read: u32,
//     pub rows_written: u32,
// }

// #[derive(Debug, Deserialize, Clone)]
// pub struct D1APIResultWrapper {
//     pub error: Option<D1APIError>,
//     pub results: Option<Vec<D1APIResult>>,
//     pub success: bool,
//     pub meta: D1APIMeta,
// }

// #[derive(Debug, Deserialize, Clone)]
// #[serde(untagged)]
// pub enum D1APIResponse {
//     SingleResponse(D1APIResultWrapper),
//     MultiResponse(Vec<D1APIResultWrapper>),
// }

// // Fake Implement Deref for D1Storage
// impl Deref for D1Storage {
//     type Target = String;

//     fn deref(&self) -> &Self::Target {
//         &self.api_token
//     }
// }

// // Fake Implement DerefMut for D1Storage
// impl DerefMut for D1Storage {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.api_token
//     }
// }

// impl D1Storage {
//     pub fn new(db_name: String, api_token: String) -> Self {
//         panic!("D1 Storage is not finished yet. Please use another storage type.");
//         Self { db_name, api_token }
//     }

//     pub async fn execute_query(
//         &self,
//         query: String,
//     ) -> Result<D1APIResultWrapper, Pin<Box<dyn Error + Send + Sync>>> {
//         println!("Executing query: {:?}", query);
//         let output = Command::new("npx")
//             .arg("wrangler")
//             .arg("d1")
//             .arg("execute")
//             .arg(self.db_name.clone())
//             .arg("--remote")
//             .arg("--json")
//             .arg("--command=".to_owned() + &query)
//             .output()
//             .expect("Failed to execute wrangler command. Check if npx wrangler is installed");

//         if !output.status.success() {
//             return Err(Pin::from(Box::from(format!(
//                 "Failed to execute wrangler command: {}",
//                 String::from_utf8_lossy(&output.stdout)
//             ))));
//         }

//         let mut result = String::from_utf8_lossy(&output.stdout).to_string();
//         println!("Result: {:?}", result);

//         let response: D1APIResponse =
//             serde_json::from_str(&result).map_err(|e| Pin::from(Box::from(e.to_string())))?;
//         match response {
//             D1APIResponse::SingleResponse(result) => Ok(result),
//             D1APIResponse::MultiResponse(results) => Ok(results[0].clone()),
//         }
//     }
// }

// impl D1Storage {
//     /// Migrates the database stored procedures.
//     async fn migrate_db(&self) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
//         //         debug!("Migrating database stored procedures");
//         //         let result = self
//         //             .execute_query(
//         //                 "
//         // CREATE PROCEDURE add_blocks_with_transactions_and_transfers (
//         //     blocks JSON,
//         //     transactions JSON,
//         //     token_transfers JSON
//         // )
//         // BEGIN
//         //     -- Insert the blocks
//         //     DECLARE block JSON;
//         //     FOR block IN SELECT value FROM json_each(blocks)
//         //     LOOP
//         //         INSERT INTO blocks (
//         //             number, hash, parent_hash, nonce, sha3_uncles, logs_bloom,
//         //         transactions_root, state_root, receipts_root, miner, difficulty,
//         //         total_difficulty, extra_data, energy_limit, energy_used, timestamp,
//         //         transaction_count, matured
//         //         ) VALUES (
//         //             block->>'block_number', block->>'block_hash', block->>'parent_hash', block->>'nonce', block->>'sha3_uncles', block->>'logs_bloom',
//         //         block->>'transactions_root', block->>'state_root', block->>'receipts_root', block->>'miner', block->>'difficulty',
//         //         block->>'total_difficulty', block->>'extra_data', block->>'energy_limit', block->>'energy_used', block->>'timestamp',
//         //         block->>'transaction_count', block->>'matured'
//         //         );
//         //     END LOOP;

//         //     -- Insert the transactions
//         //     DECLARE transaction JSON;
//         //     FOR transaction IN SELECT value FROM json_each(transactions)
//         //     LOOP
//         //         INSERT INTO transactions (
//         //             block_number, hash, nonce, block_hash, transaction_index,
//         //             from_addr, to_addr, value, energy_price, energy, input
//         //         ) VALUES (
//         //             transaction->>'block_number', transaction->>'hash', transaction->>'nonce',
//         //             transaction->>'block_hash', transaction->>'transaction_index',
//         //             transaction->>'from_addr', transaction->>'to_addr',
//         //             transaction->>'value', transaction->>'energy_price',
//         //             transaction->>'energy', transaction->>'input'
//         //         );
//         //     END LOOP;

//         //     -- Insert the token transfers
//         //     DECLARE table_name TEXT;
//         //     DECLARE transfer JSON;
//         //     FOR table_name, transfer IN SELECT key, value FROM json_each(token_transfers)
//         //     LOOP
//         //         FOR transfer IN SELECT value FROM json_each(transfer)
//         //         LOOP
//         //             EXECUTE IMMEDIATE '
//         //                 INSERT INTO ' || table_name || ' (
//         //                     tx_hash, address, from_addr,
//         //                     to_addr, value
//         //                 ) VALUES (
//         //                     transfer->>'tx_hash',
//         //                     transfer->>'address',
//         //                     transfer->>'from_addr',
//         //                     transfer->>'to_addr',
//         //                     transfer->>'value'
//         //                 );
//         //             ';
//         //         END LOOP;
//         //     END LOOP;
//         // END;"
//         //                     .to_string(),
//         //             )
//         //             .await?;
//         //         debug!("Create transactions table result: {:?}", result);

//         Ok(())
//     }

//     async fn login(&self) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
//         env::set_var("CLOUDFLARE_API_TOKEN", self.api_token.clone());
//         Ok(())
//     }

//     async fn add_block_with_transactions_and_transfers(
//         &self,
//         blocks_json: serde_json::Value,
//         transactions_json: serde_json::Value,
//         token_transfers_json: serde_json::Value,
//     ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
//         let query =
//             sqlx::query::<sqlx::Sqlite>("CALL add_block_with_transactions_and_transfers (?, ?, ?)")
//                 .bind(blocks_json)
//                 .bind(transactions_json)
//                 .bind(token_transfers_json)
//                 .sql();
//         let result = self.execute_query(query.to_string()).await?;
//         debug!(
//             "Added block with transactions and transfers: {:?}",
//             result.meta.rows_written
//         );
//         if result.error.is_some() {
//             Err(Pin::from(Box::from(
//                 result.error.unwrap().notes[0].text.clone(),
//             )))
//         } else {
//             Ok(())
//         }
//     }
// }

// #[async_trait]
// impl Storage for D1Storage {
//     /// Migrate the database.
//     async fn prepare_db(&self) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
//         self.login().await?;
//         self.migrate_db().await?;
//         Ok(())
//     }

//     async fn create_indexes(&self) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
//         // Indexes
//         // self.execute_query(
//         //     "CREATE INDEX IF NOT EXISTS idx_blocks_hash ON blocks(hash);".to_string(),
//         // )
//         // .await?;
//         // self.execute_query(
//         //     "CREATE INDEX IF NOT EXISTS idx_transactions_block_hash ON transactions(block_hash);"
//         //         .to_string(),
//         // )
//         // .await?;

//         // self.execute_query(
//         //     "CREATE INDEX IF NOT EXISTS  idx_transactions_from_addr ON transactions(from_addr);"
//         //         .to_string(),
//         // )
//         // .await?;

//         // self.execute_query(
//         //     "CREATE INDEX IF NOT EXISTS idx_transactions_to_addr ON transactions(to_addr);"
//         //         .to_string(),
//         // )
//         // .await?;

//         Ok(())
//     }

//     // async fn add_block(&mut self, block: Block) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
//     //     debug!("Adding block to db: {:?}", block.number);
//     //     let response = self.execute_query(format!("INSERT INTO blocks (number, hash, parent_hash, nonce, sha3_uncles, logs_bloom, transactions_root, state_root, receipts_root, miner, difficulty, total_difficulty, extra_data, energy_limit, energy_used, timestamp, transaction_count, matured) VALUES ({}, '{}' , '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', {}, {}, {}, {}, {})",
//     //         block.number,
//     //         block.hash,
//     //         block.parent_hash,
//     //         block.nonce,
//     //         block.sha3_uncles,
//     //         block.logs_bloom,
//     //         block.transactions_root,
//     //         block.state_root,
//     //         block.receipts_root,
//     //         block.miner,
//     //         block.difficulty,
//     //         block.total_difficulty,
//     //         block.extra_data,
//     //         block.energy_limit,
//     //         block.energy_used,
//     //         block.timestamp,
//     //         block.transaction_count,
//     //         block.matured,
//     // ))
//     //         .await?;
//     //     debug!("Added block to db: {:?}", block.number);
//     //     println!("Added block to db: {:?}", response);
//     //     if response.error.is_some() {
//     //         Err(Pin::from(Box::from(
//     //             response.error.unwrap().notes[0].text.clone(),
//     //         )))
//     //     } else {
//     //         Ok(())
//     //     }
//     // }

//     // async fn add_block_with_replace(
//     //     &mut self,
//     //     block: Block,
//     // ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
//     //     debug!("Replacing block to db: {:?}", block.number);
//     //     let response = self.execute_query(format!("REPLACE INTO blocks (number, hash, parent_hash, nonce, sha3_uncles, logs_bloom, transactions_root, state_root, receipts_root, miner, difficulty, total_difficulty, extra_data, energy_limit, energy_used, timestamp, transaction_count, matured) VALUES ({}, '{}' , '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', {}, {}, {}, {}, {})",
//     //         block.number,
//     //         block.hash,
//     //         block.parent_hash,
//     //         block.nonce,
//     //         block.sha3_uncles,
//     //         block.logs_bloom,
//     //         block.transactions_root,
//     //         block.state_root,
//     //         block.receipts_root,
//     //         block.miner,
//     //         block.difficulty,
//     //         block.total_difficulty,
//     //         block.extra_data,
//     //         block.energy_limit,
//     //         block.energy_used,
//     //         block.timestamp,
//     //         block.transaction_count,
//     //         block.matured,
//     // )).await?;
//     //     debug!("Replaced block in db: {:?}", block.number);
//     //     if response.error.is_some() {
//     //         Err(Pin::from(Box::from(
//     //             response.error.unwrap().notes[0].text.clone(),
//     //         )))
//     //     } else {
//     //         Ok(())
//     //     }
//     // }

//     async fn get_all_blocks(&self) -> Result<Vec<Block>, Pin<Box<dyn Error + Send + Sync>>> {
//         debug!("Getting all blocks from db");
//         let response = self
//             .execute_query("SELECT * FROM blocks".to_string())
//             .await?;
//         if response.error.is_some() {
//             return Err(Pin::from(Box::from(
//                 response.error.unwrap().notes[0].text.clone(),
//             )));
//         }
//         let blocks = response
//             .results
//             .unwrap()
//             .iter()
//             .map(|r| {
//                 if let D1APIResult::Block(block) = r {
//                     block.clone()
//                 } else {
//                     panic!("Unexpected result type")
//                 }
//             })
//             .collect();
//         Ok(blocks)
//     }

//     /// Returns a list of blocks in the specified range.
//     /// if end is negative, it will return all blocks from start to the latest block.
//     async fn get_blocks_in_range(
//         &self,
//         start: i64,
//         end: i64,
//     ) -> Result<Vec<Block>, Pin<Box<dyn Error + Send + Sync>>> {
//         debug!("Getting blocks in range: {:?} to {:?}", start, end);

//         let mut query = format!("SELECT * FROM blocks WHERE number >= {start} AND number <= {end}");
//         if end < 0 {
//             query = format!("SELECT * FROM blocks WHERE number >= {start}");
//         }
//         let blocks = self.execute_query(query).await?;
//         if blocks.error.is_some() {
//             return Err(Pin::from(Box::from(
//                 blocks.error.unwrap().notes[0].text.clone(),
//             )));
//         }
//         let blocks = blocks
//             .results
//             .unwrap()
//             .iter()
//             .map(|r| {
//                 if let D1APIResult::Block(block) = r {
//                     block.clone()
//                 } else {
//                     panic!("Unexpected result type")
//                 }
//             })
//             .collect();
//         Ok(blocks)
//     }

//     async fn get_block_by_number(
//         &self,
//         block_number: i64,
//     ) -> Result<Block, Pin<Box<dyn Error + Send + Sync>>> {
//         debug!("Getting block by number: {:?}", block_number);

//         let response = self
//             .execute_query(format!(
//                 "SELECT * FROM blocks WHERE number = {block_number}"
//             ))
//             .await?;
//         if response.error.is_some() {
//             return Err(Pin::from(Box::from(
//                 response.error.unwrap().notes[0].text.clone(),
//             )));
//         }
//         let block: Vec<Block> = response
//             .results
//             .unwrap()
//             .iter()
//             .map(|r| {
//                 if let D1APIResult::Block(block) = r {
//                     block.clone()
//                 } else {
//                     panic!("Unexpected result type")
//                 }
//             })
//             .collect();
//         Ok(block[0].clone())
//     }

//     async fn get_block_by_hash(
//         &self,
//         block_hash: String,
//     ) -> Result<Block, Pin<Box<dyn Error + Send + Sync>>> {
//         debug!("Getting block by hash: {:?}", block_hash);

//         let response = self
//             .execute_query(format!("SELECT * FROM blocks WHERE hash = '{block_hash}'"))
//             .await?;
//         if response.error.is_some() {
//             return Err(Pin::from(Box::from(
//                 response.error.unwrap().notes[0].text.clone(),
//             )));
//         }
//         let block: Vec<Block> = response
//             .results
//             .unwrap()
//             .iter()
//             .map(|r| {
//                 if let D1APIResult::Block(block) = r {
//                     block.clone()
//                 } else {
//                     panic!("Unexpected result type")
//                 }
//             })
//             .collect();
//         Ok(block[0].clone())
//     }

//     async fn get_latest_block_number(&self) -> Result<i64, Pin<Box<dyn Error + Send + Sync>>> {
//         debug!("Getting last block number");

//         let result = self
//             .execute_query("SELECT * FROM blocks ORDER BY number DESC LIMIT 1".to_string())
//             .await?;
//         if result.error.is_some() {
//             return Err(Pin::from(Box::from(
//                 result.error.unwrap().notes[0].text.clone(),
//             )));
//         }
//         let blocks: Vec<Block> = result
//             .results
//             .unwrap()
//             .iter()
//             .map(|r| {
//                 if let D1APIResult::Block(block) = r {
//                     block.clone()
//                 } else {
//                     panic!("Unexpected result type")
//                 }
//             })
//             .collect();

//         if blocks.len() == 0 {
//             return Ok(0);
//         }
//         Ok(blocks[0].number)
//     }

//     async fn update_blocks_to_matured(
//         &self,
//         block_height: i64,
//     ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
//         debug!("Updating blocks to matured");

//         let result = self
//             .execute_query(format!(
//                 "UPDATE blocks SET matured = 1 WHERE number <= {block_height}"
//             ))
//             .await?;
//         debug!("Updated matured blocks: {:?}", result.meta.rows_written);
//         Ok(())
//     }
//     // async fn add_transactions(
//     //     &mut self,
//     //     transactions: Vec<Transaction>,
//     // ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
//     //     if transactions.len() == 0 {
//     //         return Ok(());
//     //     }
//     //     debug!("Adding transactions to db");

//     //     let mut sql = String::from("INSERT INTO transactions (hash, nonce, block_hash, block_number, transaction_index, from_addr, to_addr, value, energy, energy_price, input) VALUES ");
//     //     let values: Vec<String> = transactions
//     //         .iter()
//     //         .map(|transaction| {
//     //             format!(
//     //                 " ('{}', '{}', '{}', {}, {}, '{}', '{}', '{}', '{}', '{}', '{}')",
//     //                 transaction.hash,
//     //                 transaction.nonce,
//     //                 transaction.block_hash,
//     //                 transaction.block_number,
//     //                 transaction.transaction_index,
//     //                 transaction.from,
//     //                 transaction.to,
//     //                 transaction.value,
//     //                 transaction.energy,
//     //                 transaction.energy_price,
//     //                 transaction.input
//     //             )
//     //         })
//     //         .collect();

//     //     sql.push_str(&values.join(","));
//     //     sql.push(';');

//     //     let response = self.execute_query(sql).await?;
//     //     debug!("Added transactions to db: {:?}", response.meta.rows_written);
//     //     if response.error.is_some() {
//     //         Err(Pin::from(Box::from(
//     //             response.error.unwrap().notes[0].text.clone(),
//     //         )))
//     //     } else {
//     //         Ok(())
//     //     }
//     // }

//     async fn get_block_transactions(
//         &self,
//         block_number: i64,
//     ) -> Result<Vec<Transaction>, Pin<Box<dyn Error + Send + Sync>>> {
//         debug!("Getting transactions for block: {:?}", block_number);

//         let response = self
//             .execute_query(format!(
//                 "SELECT * FROM transactions WHERE block_number = {block_number}"
//             ))
//             .await?;
//         if response.error.is_some() {
//             return Err(Pin::from(Box::from(
//                 response.error.unwrap().notes[0].text.clone(),
//             )));
//         }
//         let transactions = response
//             .results
//             .unwrap()
//             .iter()
//             .map(|r| {
//                 if let D1APIResult::Transaction(transaction) = r {
//                     transaction.clone()
//                 } else {
//                     panic!("Unexpected result type")
//                 }
//             })
//             .collect();
//         Ok(transactions)
//     }

//     async fn get_transaction_by_hash(
//         &self,
//         hash: String,
//     ) -> Result<Transaction, Pin<Box<dyn Error + Send + Sync>>> {
//         debug!("Getting transaction by hash: {:?}", hash);
//         let response = self
//             .execute_query(format!("SELECT * FROM transactions WHERE hash = '{hash}'"))
//             .await?;
//         if response.error.is_some() {
//             return Err(Pin::from(Box::from(
//                 response.error.unwrap().notes[0].text.clone(),
//             )));
//         }
//         let transaction: Vec<Transaction> = response
//             .results
//             .unwrap()
//             .iter()
//             .map(|r| {
//                 if let D1APIResult::Transaction(transaction) = r {
//                     transaction.clone()
//                 } else {
//                     panic!("Unexpected result type")
//                 }
//             })
//             .collect();
//         Ok(transaction[0].clone())
//     }

//     async fn create_token_transfers_tables(
//         &self,
//         tokens: HashMap<String, String>,
//     ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
//         for (token, address) in tokens {
//             let table_name = format!("{}_{}_transfers", token, &address[..8]);
//             let result = self.execute_query(format!("CREATE TABLE IF NOT EXISTS {table_name} (from_addr TEXT NOT NULL, to_addr TEXT NOT NULL, value TEXT NOT NULL, tx_hash TEXT NOT NULL, address TEXT NOT NULL);")).await?;
//             debug!("Create token transfers table result: {:?}", result);
//         }
//         Ok(())
//     }

//     async fn get_token_transfers(
//         &self,
//         token_address: String,
//         from: Option<String>,
//         to: Option<String>,
//     ) -> Result<Vec<TokenTransfer>, Pin<Box<dyn Error + Send + Sync>>> {
//         debug!("Getting token transfers for token: {:?}", token_address);
//         // Step 1: Get all table names that end with '_transfers'
//         let selector = &token_address[..8];
//         let tables_res = self.execute_query(format!("SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '%_{selector}_transfers'")).await?;
//         let table_name: String = tables_res
//             .results
//             .unwrap()
//             .first()
//             .map(|r| {
//                 if let D1APIResult::SQLTable { name } = r {
//                     name.clone()
//                 } else {
//                     panic!("Unexpected result type")
//                 }
//             })
//             .unwrap();

//         let mut query = format!("SELECT * FROM {table_name} WHERE 1 = 1");
//         if let Some(from) = from {
//             query += format!(" AND from_addr = '{from}'").as_str();
//         }
//         if let Some(to) = to {
//             query += format!(" AND to_addr = '{to}'").as_str();
//         }
//         let token_transfers = self.execute_query(query).await?;
//         if token_transfers.error.is_some() {
//             return Err(Pin::from(Box::from(
//                 token_transfers.error.unwrap().notes[0].text.clone(),
//             )));
//         }
//         let token_transfers = token_transfers
//             .results
//             .unwrap()
//             .iter()
//             .map(|r| {
//                 if let D1APIResult::TokenTransfer(token_transfer) = r {
//                     token_transfer.clone()
//                 } else {
//                     panic!("Unexpected result type")
//                 }
//             })
//             .collect();
//         Ok(token_transfers)
//     }

//     async fn get_transaction_token_transfers(
//         &self,
//         tx_hash: String,
//     ) -> Result<Vec<TokenTransfer>, Pin<Box<dyn Error + Send + Sync>>> {
//         debug!("Getting token transfers for transaction: {:?}", tx_hash);
//         // Step 1: Get all table names that end with '_transfers'
//         let stmt = self
//             .execute_query(format!(
//                 "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '%_transfers'"
//             ))
//             .await?;

//         let table_names: Vec<String> = stmt
//             .results
//             .unwrap()
//             .iter()
//             .map(|r| {
//                 if let D1APIResult::SQLTable { name } = r {
//                     name.clone()
//                 } else {
//                     panic!("Unexpected result type")
//                 }
//             })
//             .collect();

//         // Step 2: Construct the UNION ALL query
//         let mut query_parts = Vec::new();
//         for table in &table_names {
//             query_parts.push(format!(
//                 "SELECT from_addr, to_addr, value, tx_hash, address FROM {table} WHERE tx_hash = '{tx_hash}'",
//             ));
//         }
//         let query = query_parts.join(" UNION ALL ");

//         // Step 3: Execute the query and fetch results
//         let token_transfers = self.execute_query(query).await?;
//         if token_transfers.error.is_some() {
//             return Err(Pin::from(Box::from(
//                 token_transfers.error.unwrap().notes[0].text.clone(),
//             )));
//         }
//         let token_transfers = token_transfers
//             .results
//             .unwrap()
//             .iter()
//             .map(|r| {
//                 if let D1APIResult::TokenTransfer(token_transfer) = r {
//                     token_transfer.clone()
//                 } else {
//                     panic!("Unexpected result type")
//                 }
//             })
//             .collect();
//         Ok(token_transfers)
//     }

//     async fn get_address_token_transfers(
//         &self,
//         address: String,
//         transfer_type: TransferType,
//     ) -> Result<Vec<TokenTransfer>, Pin<Box<dyn Error + Send + Sync>>> {
//         debug!("Getting token transfers for address: {:?}", address);
//         // Step 1: Get all table names that end with '_transfers'
//         let stmt = self
//             .execute_query(
//                 "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '%_transfers'"
//                     .to_string(),
//             )
//             .await?;

//         let table_names: Vec<String> = stmt
//             .results
//             .unwrap()
//             .iter()
//             .map(|r| {
//                 if let D1APIResult::SQLTable { name } = r {
//                     name.clone()
//                 } else {
//                     panic!("Unexpected result type")
//                 }
//             })
//             .collect();

//         // Step 2: Construct the UNION ALL query
//         let mut query_parts = Vec::new();
//         for table in &table_names {
//             let mut subquery = format!("SELECT * FROM {} WHERE ", table);
//             match transfer_type {
//                 TransferType::From => subquery += format!("from_addr = '{address}'").as_str(),
//                 TransferType::To => subquery += format!("to_addr = '{address}'").as_str(),
//                 TransferType::All => {
//                     subquery += format!("from_addr = '{address}' OR to_addr = '{address}'").as_str()
//                 }
//             }

//             query_parts.push(subquery);
//         }
//         let query = query_parts.join(" UNION ALL ");

//         // Step 3: Execute the query and fetch results
//         let token_transfers = self.execute_query(query).await?;
//         if token_transfers.error.is_some() {
//             return Err(Pin::from(Box::from(
//                 token_transfers.error.unwrap().notes[0].text.clone(),
//             )));
//         }
//         let token_transfers = token_transfers
//             .results
//             .unwrap()
//             .iter()
//             .map(|r| {
//                 if let D1APIResult::TokenTransfer(token_transfer) = r {
//                     token_transfer.clone()
//                 } else {
//                     panic!("Unexpected result type")
//                 }
//             })
//             .collect();
//         Ok(token_transfers)
//     }

//     // async fn add_blocks(
//     //     &mut self,
//     //     blocks: Vec<Block>,
//     // ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
//     //     let mut query = format!(
//     //         "INSERT INTO blocks (number, hash, parent_hash, nonce, sha3_uncles, logs_bloom, transactions_root, state_root, receipts_root, miner, difficulty, total_difficulty, extra_data, energy_limit, energy_used, timestamp, transaction_count, matured) VALUES"
//     //     );

//     //     let values: Vec<String> = blocks
//     //         .iter()
//     //         .map(|block| {
//     //             format!(
//     //                 " ({}, '{}' , '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', {}, {}, {}, {}, {})",
//     //                 block.number,
//     //                 block.hash,
//     //                 block.parent_hash,
//     //                 block.nonce,
//     //                 block.sha3_uncles,
//     //                 block.logs_bloom,
//     //                 block.transactions_root,
//     //                 block.state_root,
//     //                 block.receipts_root,
//     //                 block.miner,
//     //                 block.difficulty,
//     //                 block.total_difficulty,
//     //                 block.extra_data,
//     //                 block.energy_limit,
//     //                 block.energy_used,
//     //                 block.timestamp,
//     //                 block.transaction_count,
//     //                 block.matured,
//     //             )
//     //         })
//     //         .collect();

//     //     query.push_str(&values.join(","));
//     //     query.push(';');

//     //     let response = self.execute_query(query).await?;
//     //     debug!("Added blocks to db: {:?}", response.meta.rows_written);

//     //     Ok(())
//     // }

//     async fn clean_block_data(
//         &self,
//         block_number: i64,
//     ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
//         debug!("Cleaning block data with transactions");

//         let result = self
//             .execute_query(format!(
//                 "DELETE FROM TokenTransfer WHERE transaction_hash IN (
//                 SELECT hash FROM Transaction WHERE block_number = {block_number}
//             )"
//             ))
//             .await?;
//         debug!(
//             "Cleaned block data with transactions: {:?}",
//             result.meta.rows_written
//         );
//         Ok(())
//     }

//     async fn insert_blocks_with_txs_and_token_transfers(
//         &self,
//         insert_all: bool,
//         blocks: &mut Vec<Block>,
//         transactions: &mut Vec<Transaction>,
//         token_transfers: &mut HashMap<String, Vec<TokenTransfer>>,
//     ) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
//         // Insert data in batches of 20
//         // if the length of the blocks or transactions is greater than 20 then
//         // slice them into batches of 20
//         if blocks.len() >= 20 || transactions.len() >= 20 || insert_all {
//             let mut blocks_json = json!({});
//             let mut transactions_json = json!({});
//             let mut token_transfers_json = json!({});

//             let mut block_batches = blocks.chunks(20);
//             while let Some(block_batch) = block_batches.next() {
//                 blocks_json = serde_json::to_value(block_batch).map_err(|e| {
//                     Pin::from(Box::from(format!("Failed to serialize blocks: {:?}", e)))
//                 })?;
//             }
//             let mut transaction_batches = transactions.chunks(20);
//             while let Some(transaction_batch) = transaction_batches.next() {
//                 transactions_json = serde_json::to_value(transaction_batch).map_err(|e| {
//                     Pin::from(Box::from(format!(
//                         "Failed to serialize transactions: {:?}",
//                         e
//                     )))
//                 })?;
//             }

//             let mut token_transfers_to_insert = HashMap::new();
//             for (table, transfers) in token_transfers.clone() {
//                 let mut transfers_batches = transfers.chunks(20);
//                 while let Some(transfers_batch) = transfers_batches.next() {
//                     token_transfers_to_insert.insert(table.to_string(), transfers_batch.to_vec());
//                 }
//             }
//             token_transfers_json =
//                 serde_json::to_value(token_transfers_to_insert).map_err(|e| {
//                     Pin::from(Box::from(format!(
//                         "Failed to serialize token transfers: {:?}",
//                         e
//                     )))
//                 })?;

//             self.add_block_with_transactions_and_transfers(
//                 blocks_json,
//                 transactions_json,
//                 token_transfers_json,
//             )
//             .await?;

//             blocks.clear();
//             transactions.clear();
//             for (_, transfers) in token_transfers {
//                 transfers.clear();
//             }
//         }
//         Ok(())
//     }

//     async fn start_cleanup_task(
//         &self,
//         _interval: tokio::time::Duration,
//         _retention_duration: tokio::time::Duration,
//     ) {
//     }
// }
