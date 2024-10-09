use std::{
    collections::{HashMap, HashSet},
    pin::Pin,
    sync::Arc,
};
use tokio::sync::Mutex;

use clap::Parser;
use config::Config;
use provider::Provider;
use storage::Storage;
use tracing::{error, info};
use types::network;

#[derive(Parser, Debug)]
pub struct ExportArgs {
    #[clap(short, long, env)]
    /// Block to start syncing from
    pub block: Option<i64>,

    #[clap(short, long, env, value_parser, num_args = 1.., value_delimiter = ',')]
    /// Watch token transfers. Provide a token type and address to watch
    /// in the format: "token_type:token_address,token_type:token_address"
    /// Example: "cbc20:cb19c7acc4c292d2943ba23c2eaa5d9c5a6652a8710c" - to watch Core Token transfers
    pub watch_tokens: Option<Vec<String>>,

    #[clap(short, long, env, value_parser, num_args = 1.., value_delimiter = ',')]
    /// Filter transactions by address. Provide a list of addresses to filter
    /// Example: "0x123,0x456,0x789"
    pub address_filter: Option<Vec<String>>,

    #[clap(short, long, env, default_value = "0")]
    /// How long to retain data in the database
    pub retention_duration: i64,

    #[clap(short, long, env, default_value = "3600")]
    /// How often to run the cleanup task. Value is in seconds
    /// Cleanup task will remove data older than retention_duration
    pub cleanup_interval: i64,
}

impl ExportArgs {
    pub async fn exec(
        &self,
        config: Config,
        provider: Provider,
        storage: Arc<dyn Storage + Send + Sync>,
    ) -> Result<(), Pin<Box<dyn std::error::Error + Sync + Send>>> {
        let network_id = provider.get_network_id().await.unwrap();
        let config = self.add_args(config, network_id);
        let mut worker = etl::ETLWorker::new(config, storage, provider).await;
        let res = worker.run().await;
        if let Err(e) = res {
            error!("Error in ETLWorker: {:?}", e);
        }
        Ok(())
    }

    pub fn add_args(&self, mut config: Config, network_id: u64) -> Config {
        config.block_number = self.block.unwrap_or_default();
        config.retention_duration = self.retention_duration;
        config.cleanup_interval = self.cleanup_interval;
        config.address_filter = self.address_filter.clone().unwrap_or_default();

        if let Some(watch_tokens) = &self.watch_tokens {
            config.watch_tokens = self.parse_watch_tokens(network_id, watch_tokens);
            info!("Monitoring token transfers: {:?}", config.watch_tokens);
        }

        config
    }

    pub fn parse_watch_tokens(
        &self,
        network_id: u64,
        watch_tokens: &Vec<String>,
    ) -> HashMap<String, HashSet<String>> {
        let mut map: HashMap<String, HashSet<String>> = HashMap::new();
        for token in watch_tokens {
            // predefined CoreToken
            if token == "ctn" {
                if network_id == 1 {
                    let mut set = HashSet::new();
                    set.insert("cb19c7acc4c292d2943ba23c2eaa5d9c5a6652a8710c".to_string());
                    map.insert("cbc20".to_string(), set);
                } else if network_id == 3 {
                    let mut set = HashSet::new();
                    set.insert("ab7935cdef94ac9e6bcbcf779277aad7025993bc1964".to_string());
                    map.insert("cbc20".to_string(), set);
                }
                continue;
            }
            let mut split = token.split(':');
            let token_type = split.next().unwrap().to_string();
            let token_address = split.next().unwrap().to_string();
            map.entry(token_type)
                .or_insert_with(HashSet::new)
                .insert(token_address);
        }
        map
    }
}
