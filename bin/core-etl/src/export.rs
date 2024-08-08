use std::{pin::Pin, sync::Arc};
use tokio::sync::Mutex;

use clap::Parser;
use config::Config;
use provider::Provider;
use storage::Storage;
use tracing::error;

#[derive(Parser, Debug)]
pub struct ExportArgs {
    #[clap(short, long, env)]
    /// Block to start syncing from
    pub block: Option<i64>,

    #[clap(short, long, env)]
    /// Continue syncing from the last block in the database
    /// This will override the block argument
    pub continue_sync: Option<bool>,

}

impl ExportArgs {
    pub async fn exec(
        &self,
        config: Config,
        provider: Provider,
        storage: Arc<Mutex<dyn Storage>>,
    ) -> Result<(), Pin<Box<dyn std::error::Error + Sync + Send>>> {
        let config = self.add_args(config);
        let mut worker = etl::ETLWorker::new(config, storage, provider).await;
        let res = worker.run().await;
        if let Err(e) = res {
            error!("Error in ETLWorker: {:?}", e);
        }
        Ok(())
    }

    pub fn add_args(&self, mut config: Config) -> Config {
        config.block_number = self.block.unwrap_or_default();
        config.continue_sync = self.continue_sync.unwrap_or_default();

        config
    }
}
