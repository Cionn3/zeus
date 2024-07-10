use std::sync::{ Arc, RwLock };
use futures_util::StreamExt;
use crossbeam::channel::Receiver;
use alloy::{
    primitives::{ address, Address, U256 },
    providers::{ Provider, RootProvider },
    pubsub::PubSubFrontend,
    rpc::types::eth::{ Block, BlockId, BlockNumberOrTag },
    sol,
};

use anyhow::anyhow;
use lazy_static::lazy_static;
lazy_static! {
    pub static ref BLOCK_ORACLE: Arc<RwLock<BlockOracle>> = BlockOracle::default();
}

use tracing::{ info, error, trace };
use super::OracleAction;

use std::time::{ Instant, Duration };

//const ETH_USD_FEED_DECIMALS: u8 = 8;

const ETH_USD_FEED: Address = address!("5f4eC3Df9cbd43714FE2740f5E3616155c5b8419");
const BNB_USD_FEED: Address = address!("0567F2323251f0Aab15c8dFb1967E4e8A7D42aeE");
const BASE_ETH_USD_FEED: Address = address!("71041dddad3595F9CEd3DcCFBe3D1F4b0a16Bb70");
const ARB_ETH_USD_FEED: Address = address!("639Fe6ab55C921f74e7fac1ee960C0B6293ba612");

/// Time out for querying the gas price
const TIME_OUT: u64 = 30;

sol!(
    #[sol(rpc)]
    contract ChainLinkOracle {
        function latestAnswer() external view returns (int256);
    }
);

/// Holds Block basic information
#[derive(Debug, Clone)]
pub struct BlockInfo {
    pub full_block: Option<Block>,
    pub number: u64,
    pub timestamp: u64,
    pub base_fee: U256,
}

impl Default for BlockInfo {
    fn default() -> Self {
        Self {
            full_block: None,
            number: 0,
            timestamp: 0,
            base_fee: U256::default(),
        }
    }
}

impl BlockInfo {
    pub fn new(full_block: Option<Block>, number: u64, timestamp: u64, base_fee: U256) -> Self {
        Self {
            full_block,
            number,
            timestamp,
            base_fee,
        }
    }

    /// Calculate the next block
    fn calc_next_block(&mut self, chain_id: u64, block: Block) -> Result<(), anyhow::Error> {
        let timestamp = match chain_id {
            1 => block.header.timestamp + 12,
            56 => block.header.timestamp + 3,
            8453 => block.header.timestamp + 2,
            42161 => block.header.timestamp, // Arbitrum??????
            _ => block.header.timestamp + 12,
        };

        let base_fee = match chain_id {
            1 => calculate_next_block_base_fee(block.clone()),
            56 => U256::from(3000000000u64), // 3 Gwei
            _ => U256::from(0), // TODO
        };

        let number = block.header.number.ok_or_else(|| anyhow!("Block number is missing"))?;

        self.number = number + 1;
        self.timestamp = timestamp;
        self.base_fee = base_fee;
        Ok(())
    }

    /// Wei to Gwei conversion
    pub fn gwei(&self) -> U256 {
        self.base_fee * U256::from(10).pow(U256::from(9))
    }

    /// Format Gwei to human readable format
    pub fn format_gwei(&self) -> String {
        format!("{:.2} Gwei", self.gwei() / U256::from(10).pow(U256::from(18)))
    }
}

#[derive(Clone)]
pub struct BlockOracle {
    pub latest_block: BlockInfo,
    pub next_block: BlockInfo,
    pub chain_id: u64,
    pub eth_price: U256,
    last_eth_price_request: Instant,
}

impl BlockOracle {
    pub async fn new(
        client: Arc<RootProvider<PubSubFrontend>>,
        chain_id: u64
    ) -> Result<Self, anyhow::Error> {
        let time = Instant::now();

        let block_id = BlockId::Number(BlockNumberOrTag::Latest);
        let block = client.get_block(block_id, true.into()).await?;
        let eth_price = get_eth_price(client.clone(), chain_id).await?;

        let block = block.ok_or_else(|| anyhow!("Block is missing"))?;

        let block_number = block.header.number.ok_or_else(|| anyhow!("Block number is missing"))?;
        let base_fee = block.header.base_fee_per_gas.ok_or_else(|| anyhow!("Base fee is missing"))?;

        let latest_block = BlockInfo::new(
            Some(block.clone()),
            block_number,
            block.header.timestamp,
            U256::from(base_fee)
        );

        let mut next_block = BlockInfo::default();
        next_block.calc_next_block(chain_id, block)?;

        info!("Block oracle initialized in {:?}ms", time.elapsed().as_millis());

        Ok(Self {
            latest_block,
            next_block,
            chain_id,
            eth_price,
            last_eth_price_request: Instant::now(),
        })
    }

    /// A default instance of the block oracle
    pub fn default() -> Arc<RwLock<Self>> {
        let block_oracle = BlockOracle {
            latest_block: BlockInfo::default(),
            next_block: BlockInfo::default(),
            chain_id: 1,
            eth_price: U256::ZERO,
            last_eth_price_request: Instant::now(),
        };

        Arc::new(RwLock::new(block_oracle))
    }

    /// Update the BlockInfo
    fn update_block_info(&mut self, block: Block) -> Result<(), anyhow::Error> {
        let number = block.header.number.ok_or_else(|| anyhow!("Block number is missing"))?;
        let base_fee = block.header.base_fee_per_gas.ok_or_else(|| anyhow!("Base fee is missing"))?;

        self.latest_block = BlockInfo::new(
            Some(block.clone()),
            number,
            block.header.timestamp,
            U256::from(base_fee)
        );

        self.next_block.calc_next_block(self.chain_id, block)?;
        trace!("Next block fee {}", self.next_block.format_gwei());
        Ok(())
    }

    pub fn latest_block(&self) -> &BlockInfo {
        &self.latest_block
    }

    pub fn next_block(&self) -> &BlockInfo {
        &self.next_block
    }

    pub fn get_eth_price(&self) -> &U256 {
        &self.eth_price
    }
}

pub async fn start_block_oracle(
    client: Arc<RootProvider<PubSubFrontend>>,
    chain_id: u64,
    oracle: Arc<RwLock<BlockOracle>>,
    receiver: Receiver<OracleAction>
) {
    trace!("Started block oracle for Chain ID: {}", chain_id);
    loop {
        let sub = client.subscribe_blocks().await;
        let mut stream = match sub {
            Ok(s) => s.into_stream(),
            Err(e) => {
                error!("Failed to subscribe to blocks: {:?}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        while let Some(block) = stream.next().await {
            match receiver.try_recv() {
                Ok(OracleAction::KILL) => {
                    trace!(
                        "Received kill signal, block oracle stopped for Chain Id: {:?}",
                        chain_id
                    );
                    return;
                }
                _ => {}
            }

            let number = if let Some(n) = block.header.number {
                n
            } else {
                error!("Block number is missing");
                continue;
            };

            trace!("Received new block {} for Chain ID: {}", number, chain_id);

            let last_request;
            {
                let mut lock = oracle.write().unwrap();

                match lock.update_block_info(block.clone()) {
                    Ok(_) => (),
                    Err(e) => error!("Failed to update block info: {:?}", e),
                }
                last_request = lock.last_eth_price_request;
            }

            let now = Instant::now();
            let timeout_expired = now.duration_since(last_request) > Duration::from_secs(TIME_OUT);

            if timeout_expired {
                let eth_price = get_eth_price(client.clone(), chain_id).await;
                match eth_price {
                    Ok(price) => {
                        let mut lock = oracle.write().unwrap();
                        lock.eth_price = price;
                        lock.last_eth_price_request = Instant::now();
                    }
                    Err(e) => error!("Failed to get ETH price: {:?}", e),
                }
            }
        }
    }
}

async fn get_eth_price(
    client: Arc<RootProvider<PubSubFrontend>>,
    chain_id: u64
) -> Result<U256, anyhow::Error> {
    let feed = match chain_id {
        1 => ETH_USD_FEED,
        56 => BNB_USD_FEED,
        8453 => BASE_ETH_USD_FEED,
        42161 => ARB_ETH_USD_FEED,
        _ => ETH_USD_FEED,
    };

    let oracle = ChainLinkOracle::new(feed, client.clone());
    let eth_usd = oracle.latestAnswer().call().await?._0;

    // convert i256 to U256
    let eth_usd = eth_usd.to_string().parse::<U256>()?;
    Ok(eth_usd)
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

/*

async fn get_gas_price(
    client: Arc<RootProvider<PubSubFrontend>>,
    chain_id: u64,
    base_fee: U256
) -> Result<(f64, U256), anyhow::Error> {
    let eth_usd = get_eth_price(client.clone(), chain_id).await?;
    let price = get_usd_value(eth_usd.clone(), base_fee)?;

    Ok((price, eth_usd))
}

fn get_usd_value(eth_usd: U256, base_fee: U256) -> Result<f64, anyhow::Error> {
    let base = U256::from(10).pow(U256::from(18));
    let value = (U256::from(base_fee) * eth_usd) / base;
    let formatted = format_units(value, ETH_USD_FEED_DECIMALS)?.parse::<f64>()?;
    Ok(formatted)
}

*/
