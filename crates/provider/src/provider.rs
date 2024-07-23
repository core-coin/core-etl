use atoms_provider::{network::Ethereum, Provider as AtomsProvider, RootProvider};
use atoms_pubsub::{PubSubFrontend, Subscription};
use atoms_rpc_client::WsConnect;
use atoms_rpc_types::BlockNumberOrTag;
use tracing::info;
use types::Block;

// pub async fn test() {
//     let ws = WsConnect::new("ws://127.0.0.1:8546");
//     let client = atoms_rpc_client::RpcClient::connect_pubsub(ws)
//         .await
//         .unwrap();
//     let provider = RootProvider::<_, Ethereum>::new(client);

//     let block = provider
//         .get_block_by_number(BlockNumberOrTag::Number(1), false)
//         .await
//         .unwrap();
//     println!("block: {:?}", block.expect("1").header.hash.unwrap());
//     let sub = provider.subscribe_blocks().await.unwrap();
//     let mut stream = sub.into_stream();
//     let mut n = 11;
//     while let Some(block) = stream.next().await {
//         println!("block: {:?}", block.header.number);
//     }
// }

pub struct Provider {
    root: RootProvider<PubSubFrontend>,
}

impl Provider {
    pub async fn new(api_url: String) -> Self {
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
}
