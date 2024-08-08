use serde::Serialize;

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize)]
pub enum Network {
    #[default]
    Mainnet,

    Devin,
}

impl Network {
    pub fn url(&self) -> String {
        match self {
            Network::Mainnet => "wss://xcbws.coreblockchain.net".to_string(),
            Network::Devin => "wss://xcbws-devin.coreblockchain.net".to_string(),
        }
    }
}
