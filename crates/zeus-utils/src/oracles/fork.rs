use tokio::sync::{ RwLock, broadcast::{ Sender, Receiver } };
use std::sync::Arc;


use alloy::{providers::{ Provider, RootProvider }, pubsub::PubSubFrontend, rpc::types::eth::{BlockId, BlockNumberOrTag }};
use revm::db::{ CacheDB, EmptyDB };
use zeus_types::forked_db::{ fork_db::ForkDB, fork_factory::ForkFactory };
use crate::new_fork_factory;

use super::block::BlockInfo;

#[derive(Debug, Clone)]
pub struct ForkOracle {
    pub fork_db: ForkDB,
    pub block: u64,
}

impl ForkOracle {
    pub async fn new(client: Arc<RootProvider<PubSubFrontend>>) -> Result<Self, anyhow::Error> {
        let fork_factory = new_fork_factory(client.clone()).await?;
        let fork_db = fork_factory.new_sandbox_fork();
        let block = client.get_block_number().await?;
        Ok(Self {
            fork_db,
            block,
        })
    }

    pub fn update_fork_db(&mut self, fork_db: ForkDB, block: u64) {
        self.fork_db = fork_db;
        self.block = block;
    }

    pub fn get_fork_db(&self) -> (ForkDB, u64) {
        (self.fork_db.clone(), self.block.clone())
    }
}

pub fn start_fork_oracle(
    client: Arc<RootProvider<PubSubFrontend>>,
    oracle: Arc<RwLock<ForkOracle>>,
    mut new_block: Receiver<BlockInfo>,
    sender: Sender<(ForkDB, u64)>
) {
    tokio::spawn(async move {
        while let Ok(block) = new_block.recv().await {
            {
                let block_id = BlockId::Number(BlockNumberOrTag::Number(block.number));
                let cache_db = CacheDB::new(EmptyDB::default());
                let fork_factory = ForkFactory::new_sandbox_factory(
                    client.clone(),
                    cache_db,
                    Some(block_id)
                );

                let fork_db = fork_factory.new_sandbox_fork();

                {
                let mut guard = oracle.write().await;
                guard.update_fork_db(fork_db.clone(), block.number.clone());
                }
                match sender.send((fork_db, block.number)) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Failed to send fork db {}", e);
                    }
                }
            }
        }
    });
}
