use std::sync::Arc;
use tokio::sync::RwLock;
use zeus_types::{ChainId, WsClient};

use crossbeam::channel::{ Sender, Receiver, bounded };
use crate::oracles::block::{start_block_oracle, BlockOracle, BlockInfo};

pub mod block;
pub mod fork;

pub enum OracleAction {
    STOP
}

/// Manage any oracles based on the provided chain_id
pub struct OracleManager {

    pub chain_id: ChainId,

    pub client: Arc<WsClient>,

    pub action_sender: Sender<OracleAction>,

    pub action_receiver: Receiver<OracleAction>,

    pub block_sender: Sender<BlockInfo>,

    pub block_receiver: Receiver<BlockInfo>,

    pub block_oracle: Arc<RwLock<BlockOracle>>,
}

impl OracleManager {
    pub async fn new(client: Arc<WsClient>, chain_id: ChainId) -> Result<Self, anyhow::Error> {
        let (action_sender, action_receiver) = bounded(10);
        let (block_sender, block_receiver) = bounded(10);
        let block_oracle = Arc::new(RwLock::new(BlockOracle::new(client.clone(), chain_id.clone()).await?));

        Ok(Self {
            chain_id,
            client,
            action_sender,
            action_receiver,
            block_sender,
            block_receiver,
            block_oracle
        })
    }

    pub fn start_block_oracle(&mut self) {
        let client = self.client.clone();
        let mut block_oracle = self.block_oracle.clone();
        let block_sender = self.block_sender.clone();
        let action_receiver = self.action_receiver.clone();
        std::thread::spawn(move || {
            start_block_oracle(client, &mut block_oracle, block_sender, action_receiver);
        });
    }

    pub fn stop_block_oracle(&self) -> Result<(), anyhow::Error> {
        match self.action_sender.send(OracleAction::STOP) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::Error::new(e))
        }
    }

}