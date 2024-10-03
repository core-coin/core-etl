use std::{collections::HashMap, pin::Pin, sync::Arc};
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

    #[clap(short, long, env, value_parser, num_args = 1.., value_delimiter = ',')]
    /// Watch token transfers. Provide a token type and address to watch
    /// in the format: "token_type:token_address,token_type:token_address"
    /// Example: "cbc20:cb19c7acc4c292d2943ba23c2eaa5d9c5a6652a8710c" - to watch Core Token transfers
    pub watch_tokens: Option<Vec<String>>,
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
        if let Some(watch_tokens) = &self.watch_tokens {
            let map = watch_tokens
                .iter()
                .map(|s| {
                    let mut split = s.split(':');
                    let token_type = split.next().unwrap().to_string();
                    let token_address = split.next().unwrap().to_string();
                    (token_type, token_address)
                })
                .collect();

            config.watch_tokens = Some(map);
        }

        config
    }
}
