use tokio::sync::RwLock;
use zeus_types::defi::dex::uniswap::pool::Pool;
use zeus_types::defi::erc20::ERC20Token;
use std::sync::Arc;
use std::str::FromStr;
use futures_util::StreamExt;
use crossbeam::channel::Receiver;
use alloy::providers::Provider;

use alloy::rpc::types::eth::{ Block, BlockId, BlockNumberOrTag };
use alloy::primitives::U256;
use zeus_types::{ WsClient, BlockInfo, defi::zeus_router::{SwapRouter, decode_swap, swap_router_bytecode} };
use crate::{dummy_account::*, ChainId, new_evm, parse_ether};
use revm::{db::{EmptyDB, CacheDB}, primitives::{TransactTo, Address}};
use zeus_types::forked_db::fork_factory::ForkFactory;
use tracing::{ info, error };
use super::OracleAction;

use std::time::{Instant, Duration};

const TIME_OUT: u64 = 60;

#[derive(Clone)]
pub struct PriceOracle {
    /// Weth price in USD
    pub weth_usdc: U256,

    pub chain_id: ChainId,

    pub weth: ERC20Token,

    pub stable_coin: ERC20Token,

    /// Time of last request
    pub last_request: Instant,
}

impl PriceOracle {
    pub async fn new(
        client: Arc<WsClient>,
        chain_id: ChainId,
        block: Block
    ) -> Result<Self, anyhow::Error> {
        let weth = ERC20Token::new(get_native_coin(chain_id.clone()), client.clone(), chain_id.id()).await?;
        let stable_coin = match chain_id {
            // USDC
            ChainId::Ethereum(_) => Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
            // USDT
            ChainId::BinanceSmartChain(_) => Address::from_str("0x55d398326f99059ff77548524499027b3197955").unwrap(),
            // USDC
            ChainId::Base(_) => Address::from_str("0x833589fcd6edb6e08f4c7c32d4f71b54bda02913").unwrap(),
            // USDC
            ChainId::Arbitrum(_) => Address::from_str("0xaf88d065e77c8cc2239327c5edb3a432268e5831").unwrap(),
        };
        let stable_coin = ERC20Token::new(stable_coin, client.clone(), chain_id.id()).await?;
        let weth_price = get_price(client.clone(), chain_id.clone(), block, weth.clone(), stable_coin.clone()).await?;

        Ok(Self {
            weth_usdc: weth_price,
            chain_id,
            weth,
            stable_coin,
            last_request: Instant::now(),
        })
    }
}

#[derive(Clone)]
pub struct BlockOracle {
    pub latest_block: BlockInfo,
    pub next_block: BlockInfo,
    pub chain_id: ChainId,
}

impl BlockOracle {
    pub async fn new(client: Arc<WsClient>, chain_id: ChainId) -> Result<Self, anyhow::Error> {
        let block = client
            .get_block(BlockId::Number(BlockNumberOrTag::Latest), true).await?
            .expect("Block is missing");

        let next_block = next_block(chain_id.clone(), block.clone())?;

        let latest_block = BlockInfo::new(
            Some(block.clone()),
            block.header.number.expect("Block number is missing"),
            block.header.timestamp,
            U256::from(block.header.base_fee_per_gas.unwrap_or_default())
        );
        

        Ok(Self {
            latest_block,
            next_block,
            chain_id,
        })
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
}

pub async fn start_block_oracle(
    client: Arc<WsClient>,
    chain_id: ChainId,
    oracle: Arc<RwLock<BlockOracle>>,
    price_oracle: Arc<RwLock<PriceOracle>>,
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

        info!("Subscribed to new blocks for the: {:?} Chain", oracle.read().await.chain_id.name());

        while let Some(block) = stream.next().await {
            info!("Received new block for the: {:?} Chain", oracle.read().await.chain_id.name());
            info!("Block Number: {}", block.header.number.expect("Block number is missing"));

            match receiver.try_recv() {
                Ok(OracleAction::STOP) => {
                    info!("Received stop signal, stopping block oracle");
                    return;
                }
                _ => {}
            }

            {
                let mut guard = oracle.write().await;
                match guard.update_block(block.clone()) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error updating block: {}", e);
                    }
                }
            }

            let weth;
            let stable_coin;
            let last_request;
            {
                let price_oracle = price_oracle.read().await;
                weth = price_oracle.weth.clone();
                stable_coin = price_oracle.stable_coin.clone();
                last_request = price_oracle.last_request;
            }

            let time = Instant::now();
            if time.duration_since(last_request) > Duration::from_secs(TIME_OUT) {
                let weth_price = match get_price(client.clone(), chain_id.clone(), block, weth, stable_coin).await {
                    Ok(weth_price) => weth_price,
                    Err(e) => {
                        error!("Error getting weth price: {}", e);
                        continue;
                    }
                };
                let mut price_oracle = price_oracle.write().await;
                price_oracle.weth_usdc = weth_price;
                price_oracle.last_request = time;
            }
    }
}
}

async fn get_price(
    client: Arc<WsClient>,
    chain_id: ChainId,
    block: Block,
    token_in: ERC20Token,
    token_out: ERC20Token
) -> Result<U256, anyhow::Error> {
    let block_id = BlockId::Number(BlockNumberOrTag::Number(block.header.number.unwrap()));

    let cache_db = CacheDB::new(EmptyDB::default());

    let mut fork_factory = ForkFactory::new_sandbox_factory(
        client.clone(),
        cache_db,
        Some(block_id)
    );

    let eoa = AccountType::EOA;
    let contract = AccountType::Contract(swap_router_bytecode());

    let dummy_caller = DummyAccount::new(eoa, parse_ether("10")?, parse_ether("10")?);
    let dummy_contract = DummyAccount::new(contract, U256::ZERO, U256::ZERO);

    if let Err(e) = insert_dummy_account(&dummy_caller, chain_id.clone(), &mut fork_factory) {
        return Err(e);
    }

    if let Err(e) = insert_dummy_account(&dummy_contract, chain_id.clone(), &mut fork_factory) {
        return Err(e);
    }

    let fork_db = fork_factory.new_sandbox_fork();

    let mut evm = new_evm(fork_db, block, chain_id.clone());

    let pool_address = match chain_id {

        // WETH/USDC V3 
        ChainId::Ethereum(_) => Address::from_str("0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640").unwrap(),
        // WBNB/USDT Pancake V3
        ChainId::BinanceSmartChain(_) => Address::from_str("0x36696169c63e42cd08ce11f5deebbcebae652050").unwrap(),
        // WETH/USDC V3
        ChainId::Base(_) => Address::from_str("0xd0b53d9277642d899df5c87a3966a349a798f224").unwrap(),
        // WETH/USDC V3
        ChainId::Arbitrum(_) => Address::from_str("0xc6962004f452be9203591991d15f6b388e09e8d0").unwrap(),
    };

    let params = SwapRouter::Params {
        input_token: token_in.address,
        output_token: token_out.address,
        amount_in: parse_ether("1")?,
        pool: pool_address,
        pool_variant: U256::from(1),
        minimum_received: U256::from(0),
    };

    // approve the contract to spend token_in
    evm.tx_mut().caller = dummy_caller.address;
    evm.tx_mut().transact_to = TransactTo::Call(token_in.address);
    evm.tx_mut().value = U256::ZERO;
    evm.tx_mut().data = token_in.encode_approve(dummy_contract.address, params.amount_in).into();

    evm.transact_commit()?;
    evm.tx_mut().transact_to = TransactTo::Call(dummy_contract.address);

    let res = evm.transact()?.result;

    let weth_price = if res.is_success() {
        decode_swap(res.into_output().unwrap())?
    } else {
        return Err(anyhow::anyhow!("Failed to get weth price"));
    };


    Ok(weth_price)

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
