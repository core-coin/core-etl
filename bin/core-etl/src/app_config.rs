use config::Config;
use types::Network;

use crate::Args;

impl Args {
    pub fn load_config(&self) -> Config {
        let mut config = Config {
            rpc_url: Network::default().url(),
            block_number: 0,
            continue_sync: false,
        };

        if self.rpc_url.is_some() {
            config.rpc_url = self.rpc_url.clone().unwrap();
        }

        if self.network.is_some() {
            config.rpc_url = self.network.clone().unwrap().url();
        }

        config
    }
}
