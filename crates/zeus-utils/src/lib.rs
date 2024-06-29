use alloy::{
    primitives::{U256, Address},
    providers::{ Provider, ProviderBuilder, RootProvider },
    pubsub::PubSubFrontend,
    rpc::types::eth::{ BlockId, BlockNumberOrTag, Block },
    transports::ws::WsConnect,
};

use revm::{ Evm, primitives::SpecId, db::{ CacheDB, EmptyDB } };
use std::sync::Arc;
use std::str::FromStr;
use anyhow::anyhow;
use bigdecimal::BigDecimal;

use zeus_types::{forked_db::{fork_db::ForkDB, fork_factory::ForkFactory}, ChainId};


pub mod dummy_account;
pub mod oracles;


pub async fn get_client(url: &str) -> Result<Arc<RootProvider<PubSubFrontend>>, anyhow::Error> {
    let client = ProviderBuilder::new().on_ws(WsConnect::new(url)).await?;
    Ok(Arc::new(client))
}

pub fn get_weth(chain_id: ChainId) -> Address {
    match chain_id {
        ChainId::Ethereum(_) => Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27ead9083C756Cc2").unwrap(),
        // WBNB
        ChainId::BinanceSmartChain(_) => Address::from_str("0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c").unwrap(),
        ChainId::Base(_) => Address::from_str("0x4200000000000000000000000000000000000006").unwrap(),
        ChainId::Arbitrum(_) => Address::from_str("0x82af49447d8a07e3bd95bd0d56f35241523fbab1").unwrap(),
    }
}

pub fn get_usdc(chain_id: ChainId) -> Address {
    match chain_id {
        ChainId::Ethereum(_) => Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
        ChainId::BinanceSmartChain(_) => Address::from_str("0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d").unwrap(),
        ChainId::Base(_) => Address::from_str("0x833589fcd6edb6e08f4c7c32d4f71b54bda02913").unwrap(),
        ChainId::Arbitrum(_) => Address::from_str("0xaf88d065e77c8cc2239327c5edb3a432268e5831").unwrap(),
    }
}

/// Creates a new Fork Enviroment
pub async fn new_fork_factory(
    client: Arc<RootProvider<PubSubFrontend>>
) -> Result<ForkFactory, anyhow::Error> {
    let latest_block = client.get_block_number().await?;
    let block_id = BlockId::Number(BlockNumberOrTag::Number(latest_block));
    let cache_db = CacheDB::new(EmptyDB::default());

    Ok(
    ForkFactory::new_sandbox_factory(
        client.clone(),
        cache_db,
        Some(block_id)
    ))
}



/// Parse the ether amount to U256
pub fn parse_ether(amount: &str) -> anyhow::Result<U256> {
    let amount = BigDecimal::from_str(amount).map_err(|e| anyhow!("Invalid number format: {}", e))?;
    let divisor = BigDecimal::from_str("1000000000000000000").expect("divisor string is invalid");

    let wei_amount = amount * divisor;

    let wei_str = wei_amount.to_string();
    let wei_str = wei_str.split('.').next().unwrap_or_default();

    // Convert the string to U256
    U256::from_str(wei_str).map_err(|e| anyhow!("Error converting to U256: {}", e))
}

/// Creates a new [Evm] instance with initial state from [ForkDB]
///
/// State changes are applied to [Evm]
pub fn new_evm(fork_db: ForkDB, block: Block, chain_id: ChainId) -> Evm<'static, (), ForkDB> {
    let spec_id = match chain_id {
        ChainId::Ethereum(_) => SpecId::CANCUN,
        ChainId::BinanceSmartChain(_) => SpecId::SHANGHAI,
        _ => SpecId::CANCUN,
    };

    let mut evm = Evm::builder().with_db(fork_db).with_spec_id(spec_id).build();

    evm.block_mut().number = U256::from(block.header.number.expect("Block number is missing"));
    evm.block_mut().timestamp = U256::from(block.header.timestamp);
    evm.block_mut().coinbase = block.header.miner;

    // Disable some checks for easier testing
    evm.cfg_mut().disable_balance_check = true;
    evm.cfg_mut().disable_block_gas_limit = true;
    evm.cfg_mut().disable_base_fee = true;
    evm
}
