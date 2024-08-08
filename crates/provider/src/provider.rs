use atoms_provider::{network::Ethereum, Provider as AtomsProvider, RootProvider};
use atoms_pubsub::{PubSubFrontend, Subscription};
use atoms_rpc_client::WsConnect;
use atoms_rpc_types::BlockNumberOrTag;
use std::marker::{Send, Sync};
use tracing::info;
use types::{Block, Transaction}; // Add this line to import the Send and Sync traits

#[derive(Debug, Clone)]
pub struct Provider {
    root: RootProvider<PubSubFrontend>,
}

impl Provider {
    pub async fn new(api_url: String) -> Self {
        info!("Connecting to provider at {}", api_url);
        let ws = WsConnect::new(api_url.clone());
        let client = atoms_rpc_client::RpcClient::connect_pubsub(ws)
            .await
            .unwrap();
        let provider: RootProvider<PubSubFrontend> = RootProvider::<_, Ethereum>::new(client);
        info!("Connected to provider at {}", api_url);
        Self { root: provider }
    }

    pub async fn subscribe_blocks(&self) -> Subscription<atoms_rpc_types::Block> {
        self.root.subscribe_blocks().await.unwrap()
    }

    pub async fn get_block(&self, query: BlockNumberOrTag) -> Option<Block> {
        let block = self.root.get_block_by_number(query, false).await.unwrap();
        block.map(|block| block.into())
    }

    pub async fn get_block_with_transactions(
        &self,
        query: BlockNumberOrTag,
    ) -> Option<(Block, Vec<Transaction>)> {
        let block = self
            .root
            .get_block_by_number(query, true)
            .await
            .unwrap()
            .unwrap();
        let txs = {
            if let Some(txs) = block.transactions.txns() {
                txs.into_iter()
                    .map(|t: &atoms_rpc_types::Transaction| t.into())
                    .collect()
            } else {
                vec![]
            }
        };
        Some((block.into(), txs))
    }
}
