use config::Config;

use crate::Args;

impl Args {
    pub fn load_config(&self) -> Config {
        Config {
            rpc_url: self.rpc_url.clone(),
        }
    }
}
