use config::Config;
use types::Network;

use crate::Args;

impl Args {
    pub fn load_config(&self) -> Config {
        let mut config = Config {
            rpc_url: Network::default().url(),
            block_number: 0,
            watch_tokens: Default::default(),
            retention_duration: 0,
            cleanup_interval: 0,
            address_filter: Default::default(),
            lazy: false,
            threads: 3,
        };

        if self.rpc_url.is_some() {
            config.rpc_url = self.rpc_url.clone().unwrap();
        }

        if self.network.is_some() {
            config.rpc_url = self.network.clone().unwrap().url();
        }

        if self.threads.is_some() {
            config.threads = self.threads.unwrap();
        }

        config
    }
}
