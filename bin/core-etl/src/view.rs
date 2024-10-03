use clap::{Parser, Subcommand};
use config::Config;
use std::{pin::Pin, sync::Arc};
use storage::Storage;
use tokio::sync::Mutex;
use tracing::info;
#[derive(Parser, Debug)]
pub struct ViewArgs {
    #[command(subcommand)]
    sub: ViewSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum ViewSubcommands {
    Block {
        #[clap(flatten)]
        group: BlockGroup,
    },
    Transaction {
        #[clap(flatten)]
        group: TransactionGroup,
    },
    TokenTransfer {
        #[clap(flatten)]
        group: TokenTransferGroup,
    },
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct BlockGroup {
    #[clap(short = 's', long, env)]
    number: Option<i64>,
    #[clap(short = 'n', long, env)]
    hash: Option<String>,
}

impl ViewArgs {
    pub async fn exec(
        &self,
        _config: Config,
        storage: Arc<Mutex<dyn Storage>>,
    ) -> Result<(), Pin<Box<dyn std::error::Error + Send + Sync>>> {
        match &self.sub {
            ViewSubcommands::Block {
                group: BlockGroup { number, hash },
            } => {
                let block = if let Some(block_number) = number {
                    storage
                        .lock()
                        .await
                        .get_block_by_number(*block_number)
                        .await?
                } else {
                    storage
                        .lock()
                        .await
                        .get_block_by_hash(hash.clone().unwrap())
                        .await?
                };
                info!("Requested block:\n {:#?}", block);
                Ok(())
            }
            ViewSubcommands::Transaction {
                group: TransactionGroup { block_number, hash },
            } => {
                if let Some(block_number) = block_number {
                    let txs = storage
                        .lock()
                        .await
                        .get_block_transctions(*block_number)
                        .await?;
                    info!("Requested transactions: {:#?}", txs);
                } else {
                    let tx = storage
                        .lock()
                        .await
                        .get_transaction_by_hash(hash.clone().unwrap())
                        .await?;
                    info!("Requested transaction: {:#?}", tx);
                };
                Ok(())
            }
            ViewSubcommands::TokenTransfer {
                group:
                    TokenTransferGroup {
                        token_address,
                        from,
                        to,
                    },
            } => {
                if let Some(address) = token_address {
                    let transfers = storage
                        .lock()
                        .await
                        .get_token_transfers(address.clone(), from.clone(), to.clone())
                        .await?;
                    info!("Requested token transfers: {:#?}", transfers);
                    return Ok(());
                }
                if let Some(from) = from {
                    let transfers = storage
                        .lock()
                        .await
                        .get_address_token_transfers(from.to_string(), types::TransferType::From)
                        .await?;
                    info!("Requested token transfers: {:#?}", transfers);
                    return Ok(());
                }
                if let Some(to) = to {
                    let transfers = storage
                        .lock()
                        .await
                        .get_address_token_transfers(to.to_string(), types::TransferType::To)
                        .await?;
                    info!("Requested token transfers: {:#?}", transfers);
                    return Ok(());
                }
                panic!("Invalid token transfer query");
            }
        }
    }
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct TransactionGroup {
    #[clap(short = 'b', long, env)]
    block_number: Option<i64>,
    #[clap(short = 'n', long, env)]
    hash: Option<String>,
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct TokenTransferGroup {
    #[clap(short = 'a', long, env)]
    token_address: Option<String>,
    #[clap(short = 'f', long, env)]
    from: Option<String>,
    #[clap(short = 't', long, env)]
    to: Option<String>,
}
