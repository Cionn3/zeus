use tokio::sync::{RwLock, broadcast::Sender};
use std::sync::Arc;
use futures_util::StreamExt;

use alloy::providers::{ Provider, RootProvider };
use alloy::pubsub::PubSubFrontend;
use alloy::rpc::types::eth::{Block, BlockId, BlockNumberOrTag};
use alloy::primitives::U256;
use crate::ChainId;


// OP 2 sec block time
// Arbitrum is not stable
// BSC 3 sec block time

/// Holds Block basic information
#[derive(Debug, Clone, Default)]
pub struct BlockInfo {
    pub number: u64,
    pub timestamp: u64,
    pub base_fee: U256
}

impl BlockInfo {
    fn new(number: u64, timestamp: u64, base_fee: U256) -> Self {
        Self {
            number,
            timestamp,
            base_fee
        }
    }


}

pub struct BlockOracle {
    pub latest_block: BlockInfo,
    pub next_block: BlockInfo,
    pub chain_id: ChainId,
}

impl BlockOracle {
    pub async fn new(client: Arc<RootProvider<PubSubFrontend>>, chain_id: ChainId) -> Result<Self, anyhow::Error> {
        let block = client.get_block(BlockId::Number(BlockNumberOrTag::Latest), true).await?.expect("Block is missing");

        let next_block = next_block(chain_id.clone(), block.clone())?;

        let latest_block = BlockInfo::new(
            block.header.number.expect("Block number is missing"),
            block.header.timestamp,
            U256::from(block.header.base_fee_per_gas.unwrap_or_default())
        );
        
        Ok(Self {
            latest_block,
            next_block,
            chain_id
        })
    }

    fn update_block(&mut self, block: Block) -> Result<(), anyhow::Error> {
        self.latest_block = BlockInfo::new(
            block.header.number.expect("Block number is missing"),
            block.header.timestamp,
            U256::from(block.header.base_fee_per_gas.unwrap_or_default())
        );

        self.next_block = next_block(self.chain_id.clone(), block)?;

        Ok(())
    }
}


pub fn start_block_oracle(
    client: Arc<RootProvider<PubSubFrontend>>,
    oracle: &mut Arc<RwLock<BlockOracle>>,
    sender: Sender<BlockInfo>,
) {
    let oracle = oracle.clone();
    tokio::spawn(async move {
        loop {
            let sub = client.subscribe_blocks().await;
            let mut stream = match sub {
                Ok(s) => s.into_stream(),
                Err(e) => {
                    println!("Error subscribing to blocks: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

            while let Some(block) = stream.next().await {
                {
                    let mut guard = oracle.write().await;
                    match guard.update_block(block.clone()) {
                        Ok(_) => {
                            let _ = sender.send(guard.latest_block.clone());
                        },
                        Err(e) => {
                            println!("Error updating block: {}", e);
                        }
                    }
                }
            }
            
        }
    });

}


/// Calculate the next block
fn next_block(chain_id: ChainId, block: Block) -> Result<BlockInfo, anyhow::Error> {
    let timestamp = match chain_id {
        ChainId::Ethereum(_) => block.header.timestamp + 12,
        ChainId::BinanceSmartChain(_) => block.header.timestamp + 3,
        ChainId::Base(_) => block.header.timestamp + 2,
        ChainId::Arbitrum(_) => block.header.timestamp + 1, // ! Arbitrum doesnt have stable block time
    };
    let base_fee = match chain_id {
        ChainId::Ethereum(_) => calculate_next_block_base_fee(block.clone()),
        ChainId::BinanceSmartChain(_) => U256::from(3000000000u64), // 3 Gwei
        _=> U256::from(0) // TODO
    };
    let number = block.header.number.expect("Block number is missing");
    Ok(BlockInfo::new(number + 1, timestamp, base_fee))
}

/// Calculate the next block base fee
// based on math provided here: https://ethereum.stackexchange.com/questions/107173/how-is-the-base-fee-per-gas-computed-for-a-new-block
fn calculate_next_block_base_fee(block: Block) -> U256 {
    // Get the block base fee per gas
    let current_base_fee_per_gas = block.header.base_fee_per_gas.unwrap_or_default();

    // Get the mount of gas used in the block
    let current_gas_used = block.header.gas_used;

    let current_gas_target = block.header.gas_limit / 2;

    if current_gas_used == current_gas_target {
        U256::from(current_base_fee_per_gas)
    } else if current_gas_used > current_gas_target {
        let gas_used_delta = current_gas_used - current_gas_target;
        let base_fee_per_gas_delta =
            (current_base_fee_per_gas * gas_used_delta) / current_gas_target / 8;

        return U256::from(current_base_fee_per_gas + base_fee_per_gas_delta);
    } else {
        let gas_used_delta = current_gas_target - current_gas_used;
        let base_fee_per_gas_delta =
            (current_base_fee_per_gas * gas_used_delta) / current_gas_target / 8;

        return U256::from(current_base_fee_per_gas - base_fee_per_gas_delta);
    }
}