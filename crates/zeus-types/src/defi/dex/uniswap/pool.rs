use alloy::{ primitives::{ Address, U256 }, providers::RootProvider, sol };
use alloy::pubsub::PubSubFrontend;
use std::sync::Arc;
use std::str::FromStr;
use lazy_static::lazy_static;
use crate::defi::erc20::ERC20Token;
use crate::ChainId;

lazy_static! {
    // Ethereum Mainnet Uniswap Factories
    static ref ETH_UNISWAP_V2_FACTORY: Address = Address::from_str(
        "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"
    ).unwrap();
    static ref ETH_UNISWAP_V3_FACTORY: Address = Address::from_str(
        "0x1F98431c8aD98523631AE4a59f267346ea31F984"
    ).unwrap();

    // Binance Smart Chain Mainnet PancakeSwap Factories
    static ref BSC_PANCAKESWAP_V2_FACTORY: Address = Address::from_str(
        "0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73"
    ).unwrap();
    static ref BSC_PANCAKESWAP_V3_FACTORY: Address = Address::from_str(
        "0x0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865"
    ).unwrap();

    // Base Mainnet Uniswap Factories
    static ref BASE_UNISWAP_V2_FACTORY: Address = Address::from_str(
        "0x8909Dc15e40173Ff4699343b6eB8132c65e18eC6"
    ).unwrap();
    static ref BASE_UNISWAP_V3_FACTORY: Address = Address::from_str(
        "0x33128a8fC17869897dcE68Ed026d694621f6FDfD"
    ).unwrap();

    // Arbitrum Mainnet Uniswap Factories
    static ref ARBITRUM_UNISWAP_V2_FACTORY: Address = Address::from_str(
        "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"
    ).unwrap();
    static ref ARBITRUM_UNISWAP_V3_FACTORY: Address = Address::from_str(
        "0x1F98431c8aD98523631AE4a59f267346ea31F984"
    ).unwrap();
}

sol! {
    #[sol(rpc)]
    contract UniswapV2Factory {
        function getPair(address tokenA, address tokenB) external view returns (address pair);
    }
    #[sol(rpc)]
    contract UniswapV3Factory {
        function getPool(address tokenA, address tokenB, uint24 fee) external view returns (address pool);
    }
}

#[derive(Debug, Clone)]
pub struct Pool {
    pub chain_id: u64,
    pub address: Address,
    pub token0: ERC20Token,
    pub token1: ERC20Token,
    pub variant: PoolVariant,
    pub fee: u32,
}

impl Pool {

    pub fn new(address: Address, token0: ERC20Token, token1: ERC20Token, variant: PoolVariant, fee:u32, chain_id: u64) -> Self {
        Self {
            chain_id,
            address,
            token0,
            token1,
            variant,
            fee
        }
    }

    pub fn variant(&self) -> U256 {
        match self.variant {
            PoolVariant::UniswapV2 => U256::ZERO,
            PoolVariant::UniswapV3 => U256::from(1),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PoolVariant {
    UniswapV2,
    UniswapV3,
}

impl PoolVariant {
    pub fn from_u256(value: U256) -> Self {
        match value {
            U256::ZERO => PoolVariant::UniswapV2,
            _ => PoolVariant::UniswapV3,
        }
    }
}

/// Gets a Uniswap V2 pool based on token0 and token1
pub async fn get_v2_pool(
    token0: ERC20Token,
    token1: ERC20Token,
    chain_id: ChainId,
    client: Arc<RootProvider<PubSubFrontend>>
) -> Result<Option<Pool>, anyhow::Error> {
    let fact_addr = get_v2_pool_factory(chain_id.clone());
    let factory = UniswapV2Factory::new(fact_addr, client.clone());
    let pair = factory.getPair(token0.address, token1.address).call().await?.pair;
    if pair == Address::ZERO {
        return Ok(None);
    }

    Ok(
        Some(Pool {
            chain_id: chain_id.id(),
            address: pair,
            token0,
            token1,
            variant: PoolVariant::UniswapV2,
            fee: 3000
        })
    )
}

/// Returns all Uniswap V3 pools based on token0 and token1
pub async fn get_v3_pools(
    token0: ERC20Token,
    token1: ERC20Token,
    chain_id: ChainId,
    client: Arc<RootProvider<PubSubFrontend>>
) -> Result<Vec<Pool>, anyhow::Error> {
    let mut pools = Vec::new();
    let fact_addr = get_v3_pool_factory(chain_id.clone());
    let factory = UniswapV3Factory::new(fact_addr, client.clone());
    for fee in &[100, 500, 3000, 10000] {
        let pool = factory.getPool(token0.address, token1.address, *fee).call().await?.pool;
        if pool != Address::ZERO {
            pools.push(Pool {
                chain_id: chain_id.id(),
                address: pool,
                token0: token0.clone(),
                token1: token1.clone(),
                variant: PoolVariant::UniswapV3,
                fee: fee.clone()
            });
        }
    }

    Ok(pools)
}

/// Gets the v2 pool factory based on the chain id
/// 
/// Supports Uniswap V2 and PancakeSwap V2
pub fn get_v2_pool_factory(chain_id: ChainId) -> Address {
    match chain_id {
        ChainId::Ethereum(_) => *ETH_UNISWAP_V2_FACTORY,
        ChainId::BinanceSmartChain(_) => *BSC_PANCAKESWAP_V2_FACTORY,
        ChainId::Base(_) => *BASE_UNISWAP_V2_FACTORY,
        ChainId::Arbitrum(_) => *ARBITRUM_UNISWAP_V2_FACTORY,
    }
}

/// Gets the v3 pool factory based on the chain id
/// 
/// Supports Uniswap V3 and PancakeSwap V3
pub fn get_v3_pool_factory(chain_id: ChainId) -> Address {
    match chain_id {
        ChainId::Ethereum(_) => *ETH_UNISWAP_V3_FACTORY,
        ChainId::BinanceSmartChain(_) => *BSC_PANCAKESWAP_V3_FACTORY,
        ChainId::Base(_) => *BASE_UNISWAP_V3_FACTORY,
        ChainId::Arbitrum(_) => *ARBITRUM_UNISWAP_V3_FACTORY,
    }
}
