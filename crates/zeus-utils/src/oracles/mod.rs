use std::sync::Arc;
use tokio::sync::RwLock;
use zeus_types::{ChainId, WsClient, BlockInfo};

use crossbeam::channel::{ Sender, Receiver, bounded };
use crate::oracles::block::{start_block_oracle, BlockOracle, PriceOracle};

pub mod block;

pub mod fork;

pub enum OracleAction {
    STOP
}

/// Manage any oracles based on the provided chain_id
#[derive(Clone)]
pub struct OracleManager {

    pub chain_id: ChainId,

    pub client: Arc<WsClient>,

    pub action_sender: Sender<OracleAction>,

    pub action_receiver: Receiver<OracleAction>,

    pub block_oracle: Arc<RwLock<BlockOracle>>,

    pub price_oracle: Arc<RwLock<PriceOracle>>,

}

impl OracleManager {
    pub async fn new(client: Arc<WsClient>, chain_id: ChainId) -> Result<Self, anyhow::Error> {
        let (action_sender, action_receiver) = bounded(10);

        let block_oracle = Arc::new(RwLock::new(BlockOracle::new(client.clone(), chain_id.clone()).await?));
        let block;
        {
            let block_oracle = block_oracle.read().await;
            block = block_oracle.latest_block.full_block.clone();
        }
        let price_oracle = Arc::new(RwLock::new(PriceOracle::new(client.clone(), chain_id.clone(), block.unwrap()).await?));

        Ok(Self {
            chain_id,
            client,
            action_sender,
            action_receiver,
            block_oracle,
            price_oracle
        })
    }

    pub async fn get_block_info(&self) -> (BlockInfo, BlockInfo) {
        let block_oracle = self.block_oracle.read().await;
        (block_oracle.latest_block.clone(), block_oracle.next_block.clone())
    }

    pub async fn start_oracles(&self) {
        let client = self.client.clone();
        let block_oracle = self.block_oracle.clone();
        let action_receiver = self.action_receiver.clone();
        let action_receiver_2 = self.action_receiver.clone();
        let price_oracle = self.price_oracle.clone();
        let chain_id = self.chain_id.clone();

        tokio::spawn(async move {
            start_block_oracle(client, chain_id, block_oracle, price_oracle, action_receiver).await;
        });
    }

    pub fn stop_block_oracle(&self) -> Result<(), anyhow::Error> {
        match self.action_sender.send(OracleAction::STOP) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::Error::new(e))
        }
    }

}