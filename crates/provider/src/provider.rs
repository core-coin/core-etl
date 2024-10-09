use atoms_provider::{network::Ethereum, Provider as AtomsProvider, RootProvider};
use atoms_pubsub::{PubSubFrontend, Subscription};
use atoms_rpc_client::{RpcClient, WsConnect};
use atoms_rpc_types::{BlockNumberOrTag, TransactionReceipt};
use base_primitives::{hex::FromHex, B256};
use std::{
    error::Error,
    marker::{Send, Sync},
    pin::Pin,
};
use tokio::time::{sleep, Duration};
use tracing::info;
use types::{Block, Transaction};

use crate::error::ProviderError; // Add this line to import the Send and Sync traits

#[derive(Debug, Clone)]
pub struct Provider {
    root: RootProvider<PubSubFrontend>,
}

impl Provider {
    pub async fn new(api_url: String) -> Self {
        // try to connect to the provider 5 times before giving up
        // wait 5 seconds between each attempt
        let mut client = RpcClient::connect_pubsub(WsConnect::new(api_url.clone())).await;
        for try_num in 0..5 {
            if client.is_ok() {
                break;
            }
            info!(
                "Connecting to provider at {}. {} try...",
                api_url,
                try_num + 1
            );
            sleep(Duration::from_secs(5)).await;
            client = RpcClient::connect_pubsub(WsConnect::new(api_url.clone())).await;
        }
        let provider: RootProvider<PubSubFrontend> =
            RootProvider::<_, Ethereum>::new(client.unwrap());
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

    pub async fn get_transaction_receipt(
        &self,
        tx_hash: String,
    ) -> Result<TransactionReceipt, Pin<Box<dyn Error + Send + Sync>>> {
        let receipt = self
            .root
            .get_transaction_receipt(B256::from_hex(tx_hash).unwrap())
            .await
            .unwrap();
        match receipt {
            Some(receipt) => Ok(receipt),
            None => Err(Box::pin(ProviderError::InvalidReceiptHash)),
        }
    }

    pub async fn get_network_id(&self) -> Result<u64, Pin<Box<dyn Error + Send + Sync>>> {
        let network_id = self.root.get_chain_id().await.unwrap();
        Ok(network_id)
    }
}
