use alloy::{
    primitives::U256,
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

use zeus_types::forked_db::{fork_db::ForkDB, fork_factory::ForkFactory};
use zeus_types::ChainId;


pub mod dummy_account;
pub mod oracles;


pub async fn get_client(url: &str) -> Result<Arc<RootProvider<PubSubFrontend>>, anyhow::Error> {
    let client = ProviderBuilder::new().on_ws(WsConnect::new(url)).await?;
    Ok(Arc::new(client))
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
