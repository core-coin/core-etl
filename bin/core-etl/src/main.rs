mod logging;
use std::{error::Error, pin::Pin};
use types::Network;

use logging::init_logging;
mod app_config;
mod app_storage;
use app_storage::StorageType;
use clap::{command, Parser, Subcommand};
use dotenvy::dotenv;
use provider::Provider;

mod view;
use view::ViewArgs;

mod export;
use export::ExportArgs;

mod verify;
use verify::VerifyArgs;

/// Commands for core-etl application
#[derive(Debug, Parser)]
#[clap(name = "core-etl", author, version, about)]
pub(crate) struct Args {
    /// URL of the RPC node that provides the blockchain data
    #[clap(short, long, env)]
    pub rpc_url: Option<String>,

    #[clap(short, long, env, value_enum)]
    /// Network to sync data from (e.g. mainnet, devin, private)
    /// If flag is set - rpc_url is not required
    pub network: Option<Network>,

    /// Storage type which is used for saving the blockchain data
    #[clap(long, env, default_value_t, value_enum)]
    pub storage: StorageType,

    #[clap(short, long, env)]
    /// Path to SQlite3 file where the blockchain data is saved
    pub sqlite3_path: Option<String>,

    #[clap(short, long, env)]
    /// Postgres database DSN where the blockchain data is saved
    pub postgres_db_dsn: Option<String>,

    #[clap(short, long, env, default_value = "etl")]
    /// Prefix for the tables in the database
    /// This is useful when running multiple instances of the ETL
    pub tables_prefix: String,

    #[clap(short, long, env, value_parser, num_args = 1.., value_delimiter = ',', default_value = "blocks,transactions,token_transfers") ]
    /// Which data will be stored in the database
    pub modules: Vec<String>,

    #[clap(long, env, default_value = "3")]
    /// Number of working threads during the initial sync
    pub threads: Option<usize>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
#[command(subcommand_help_heading = "Core Blockchain ETL (Extract, Transform, Load) tool")]
pub enum Commands {
    /// Export blockchain data to storage
    #[command(subcommand_help_heading = "Export data")]
    Export(ExportArgs),

    /// View blockchain data from storage
    #[command(subcommand_help_heading = "View data")]
    View(ViewArgs),

    /// Verify blockchain data in storage
    #[command(subcommand_help_heading = "Verify data")]
    Verify(VerifyArgs),
}

impl Args {
    pub(crate) async fn exec(&self) -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
        let config = self.load_config();
        let storage = self.choose_storage().await;

        match &self.command {
            Commands::Export(export_args) => {
                let provider = Provider::new(config.rpc_url.clone()).await;
                export_args.exec(config, provider, storage).await
            }
            Commands::Verify(verify_args) => {
                let provider = Provider::new(config.rpc_url.clone()).await;
                verify_args.exec(config, provider, storage).await
            }
            Commands::View(view_args) => view_args.exec(config, storage).await,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Pin<Box<dyn Error + Send + Sync>>> {
    init_logging();
    dotenv().ok();

    let cmd = Args::parse();
    cmd.exec().await
}
