mod logging;
use std::error::Error;

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

/// Commands for core-etl application
#[derive(Debug, Parser)]
#[clap(name = "core-etl", author, version, about)]
pub(crate) struct Args {
    /// URL of the RPC node that provides the blockchain data
    #[clap(short, long, env, default_value = "ws://127.0.0.1:8546")]
    pub rpc_url: String,

    /// Storage type which is used for saving the blockchain data
    #[clap(long, env, default_value_t, value_enum)]
    pub storage: StorageType,

    #[clap(short, long, env)]
    /// URL of the storage where the blockchain data is saved
    pub storage_url: String,

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
}

impl Args {
    pub(crate) async fn exec(&self) -> Result<(), Box<dyn Error>> {
        let config = self.load_config();
        let storage = self.choose_storage().await;

        match &self.command {
            Commands::Export(export_args) => {
                let provider = Provider::new(self.rpc_url.clone()).await;
                export_args.exec(config, provider, storage).await
            }
            Commands::View(view_args) => view_args.exec(config, storage).await,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_logging();
    dotenv().ok();

    let cmd = Args::parse();
    cmd.exec().await
}
