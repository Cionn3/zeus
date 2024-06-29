use std::sync::{ Arc, RwLock };
use futures_util::StreamExt;
use crossbeam::channel::Receiver;
use alloy::{
    primitives::{ address, utils::format_units, Address, U256 },
    providers::Provider,
    rpc::types::eth::{ Block, BlockId, BlockNumberOrTag },
    sol,
};

use zeus_types::{ WsClient, BlockInfo };
use crate::ChainId;
use tracing::{ info, error };
use super::OracleAction;

use std::time::{ Instant, Duration };

const ETH_USD_FEED: Address = address!("5f4eC3Df9cbd43714FE2740f5E3616155c5b8419");
const ETH_USD_FEED_DECIMALS: u8 = 8;

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



/// Gas Price in USD
#[derive(Clone)]
struct GasPrice {
    price: f64,
    eth_usd: U256,
    last_request: Instant,
}

impl GasPrice {
    async fn new(
        client: Arc<WsClient>,
        chain_id: u64,
        base_fee: U256
    ) -> Result<Self, anyhow::Error> {
        let eth_usd = get_eth_price(client.clone(), chain_id).await?;
        let price = get_usd_value(eth_usd.clone(), base_fee)?;

        Ok(Self { price, eth_usd, last_request: Instant::now() })
    }

    fn update(&mut self, price: f64, eth_usd: U256) {
        self.price = price;
        self.eth_usd = eth_usd;
        self.last_request = Instant::now();
    }
}

#[derive(Clone)]
pub struct BlockOracle {
    pub latest_block: BlockInfo,
    pub next_block: BlockInfo,
    pub chain_id: ChainId,
    gas_price: GasPrice,
}

impl BlockOracle {
    pub async fn new(client: Arc<WsClient>, chain_id: ChainId) -> Result<Self, anyhow::Error> {
        let time = Instant::now();

        let block = client
            .get_block(BlockId::Number(BlockNumberOrTag::Latest), true.into()).await?
            .expect("Block is missing");

        let next_block = next_block(chain_id.clone(), block.clone())?;

        let latest_block = BlockInfo::new(
            Some(block.clone()),
            block.header.number.expect("Block number is missing"),
            block.header.timestamp,
            U256::from(block.header.base_fee_per_gas.unwrap_or_default())
        );

        let gas_price = GasPrice::new(client.clone(), chain_id.id(), next_block.base_fee).await?;

        info!("Block Oracle initialized in: {:?}ms", time.elapsed().as_millis());

        Ok(Self {
            latest_block,
            next_block,
            chain_id,
            gas_price,
        })
    }

    pub fn default() -> Arc<RwLock<Self>> {
        let latest_block = BlockInfo {
            full_block: None,
            number: 0,
            timestamp: 0,
            base_fee: U256::from(0),
        };

        let next_block = BlockInfo::new(None, 1, 1, U256::from(0));

        let gas_price = GasPrice {
            price: 0.0,
            eth_usd: U256::from(0),
            last_request: Instant::now(),
        };

        Arc::new(
            RwLock::new(Self {
                latest_block,
                next_block,
                chain_id: ChainId::Ethereum(1),
                gas_price,
            })
        )
    }

    fn update_block(&mut self, block: Block) -> Result<(), anyhow::Error> {
        self.latest_block = BlockInfo::new(
            Some(block.clone()),
            block.header.number.expect("Block number is missing"),
            block.header.timestamp,
            U256::from(block.header.base_fee_per_gas.unwrap_or_default())
        );

        self.next_block = next_block(self.chain_id.clone(), block)?;

        Ok(())
    }

    pub fn get_block_info(&self) -> (BlockInfo, BlockInfo) {
        (self.latest_block.clone(), self.next_block.clone())
    }

    pub fn get_gas_price(&self) -> f64 {
        self.gas_price.price.clone()
    }

    pub fn get_eth_usd(&self) -> U256 {
        self.gas_price.eth_usd.clone()
    }
}

pub async fn start_block_oracle(
    client: Arc<WsClient>,
    chain_id: ChainId,
    oracle: Arc<RwLock<BlockOracle>>,
    receiver: Receiver<OracleAction>
) {
    loop {
        let sub = client.subscribe_blocks().await;
        let mut stream = match sub {
            Ok(s) => s.into_stream(),
            Err(e) => {
                error!("Error subscribing to blocks: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        let chain_name = chain_id.name();
        while let Some(block) = stream.next().await {
            info!("Received new block for the: {:?} Chain", chain_name);
            info!("Block Number: {}", block.header.number.expect("Block number is missing"));

            match receiver.try_recv() {
                Ok(OracleAction::STOP) => {
                    info!("Received stop signal, stopping block oracle for: {:?}", chain_name);
                    return;
                }
                _ => {}
            }

            {
                let mut guard = oracle.write().unwrap();
                match guard.update_block(block.clone()) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error updating block: {}", e);
                    }
                }
            }

            let last_request;
            let base_fee;

            {
                let oracle = oracle.read().unwrap();
                last_request = oracle.gas_price.last_request;
                base_fee = oracle.next_block.base_fee;
            }

            let now = Instant::now();
            if now.duration_since(last_request) > Duration::from_secs(TIME_OUT) {
                let (gas_price, eth_usd) = match
                    get_gas_price(client.clone(), chain_id.id(), base_fee).await
                {
                    Ok((price, eth_usd)) => (price, eth_usd),
                    Err(e) => {
                        error!("Error getting gas price: {}", e);
                        continue;
                    }
                };

                let mut oracle = oracle.write().unwrap();
                oracle.gas_price.update(gas_price, eth_usd);
                info!("Gas Price updated: ${}", oracle.gas_price.price.clone());
            }
        }
    }
}

async fn get_gas_price(
    client: Arc<WsClient>,
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

async fn get_eth_price(
    client: Arc<WsClient>,
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
        _ => U256::from(0), // TODO
    };
    let number = block.header.number.expect("Block number is missing");
    Ok(BlockInfo::new(None, number + 1, timestamp, base_fee))
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
