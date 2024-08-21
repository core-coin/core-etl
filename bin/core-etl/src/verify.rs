use atoms_rpc_types::BlockNumberOrTag;
use clap::{Parser, Subcommand};
use config::Config;
use provider::Provider;
use std::sync::Arc;
use std::{fmt::Error as fmt_err, pin::Pin};
use storage::Storage;
use tokio::sync::Mutex;
use tracing::{error, info};

#[derive(Parser, Debug)]
pub struct VerifyArgs {
    #[command(subcommand)]
    sub: VerifySubcommands,
}

#[derive(Subcommand, Debug)]
pub enum VerifySubcommands {
    Blocks {
        #[clap(short, long, env)]
        /// Block to start checking from
        block: Option<i64>,
    },
    Transactions {},
}

impl VerifyArgs {
    pub async fn exec(
        &self,
        _config: Config,
        provider: Provider,
        storage: Arc<Mutex<dyn Storage>>,
    ) -> Result<(), Pin<Box<dyn std::error::Error + Send + Sync>>> {
        match &self.sub {
            VerifySubcommands::Blocks { block } => {
                let rpc_block = provider
                    .get_block(BlockNumberOrTag::Latest)
                    .await
                    .unwrap()
                    .number;

                // check blocks from the argument number
                if block.is_some() {
                    let mut blocks = storage
                        .lock()
                        .await
                        .get_blocks_in_range(block.unwrap_or_default(), -1)
                        .await?;
                    blocks.sort_by(|a, b| a.number.cmp(&b.number));

                    for (i, block) in blocks.iter().enumerate() {
                        if i != blocks.len() - 1 {
                            let next_block = blocks.get(i + 1).unwrap();
                            if block.number != next_block.number - 1 {
                                error!(
                                    "Error: Incorrect block number sequence: {} and {}",
                                    block.number, next_block.number
                                );
                                return Err(Box::pin(fmt_err));
                            }
                        }
                    }
                    info!(
                        "DB is synced from block {} to block {}",
                        blocks.first().unwrap().number,
                        blocks.last().unwrap().number
                    );
                    info!("All blocks are in order");
                    info!("Last block in blockchain: {}", rpc_block);
                    return Ok(());
                }

                // check blocks from 0 to the latest block
                let mut blocks = storage.lock().await.get_all_blocks().await?;
                blocks.sort_by(|a, b| a.number.cmp(&b.number));

                for (i, block) in blocks.iter().enumerate() {
                    if block.number != i as i64 {
                        error!("Error: Block {} not found in DB", i);
                        return Err(Box::pin(fmt_err));
                    }
                }

               info!(
                   last_synced_block = blocks.last().unwrap().number,
                   "DB is synced"
               );
               info!(
                   "All blocks are in order"
               );
               info!(
                   last_block_in_chain = %rpc_block,
                   "Last block in blockchain"
               );
                Ok(())
            }
            VerifySubcommands::Transactions {} => Ok(()),
        }
    }
}
