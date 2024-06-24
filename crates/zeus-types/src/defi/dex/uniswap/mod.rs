use alloy::{
    primitives::{ Address, Bytes, U256 },
    providers::RootProvider,
    sol,
};
use std::{sync::Arc, str::FromStr};
use crate::{ChainId, WsClient};


pub mod pool;


sol! {
    #[sol(rpc)]
    contract V3Quoter {
        function quoteExactInput(bytes memory path, uint256 amountIn) external override returns (uint256 amountOut);
}
    #[sol(rpc)]
    contract UniswapV2Pair {
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
    }
}


pub async fn get_v3_quote(
    client: Arc<WsClient>,
    chain_id: ChainId,
    path: Vec<Address>,
    amount_in: U256,
) -> Result<U256, anyhow::Error> {
    let address = get_quoter_address(chain_id);
    let contract = V3Quoter::new(address, client.clone());

    let mut path_bytes = Vec::new();

    for address in path {
        let addr = address.as_slice();
        path_bytes.extend_from_slice(addr);
    }

    let path = Bytes::from(path_bytes);

    let amount_out = contract.quoteExactInput(path, amount_in).call().await?.amountOut;
    Ok(amount_out)
}

pub async fn get_v2_quote(
    client: Arc<WsClient>,
    pool: Address,
    token_in: Address,
    token_out: Address,
    amount_in: U256,
) -> Result<U256, anyhow::Error> {
    let pair = UniswapV2Pair::new(pool, client.clone());
    let reserves = pair.getReserves().call().await?;
    let reserve0 = reserves.reserve0;
    let reserve1 = reserves.reserve1;

    let amount_out = get_amount_out(token_in, token_out, (reserve0, reserve1), amount_in);
    Ok(amount_out)
}

fn get_amount_out(
    token_in: Address,
    token_out: Address,
    reserves: (u128, u128),
    amount_in: U256,
) -> U256 {
    let (reserve_in, reserve_out) = if token_in < token_out {
        (U256::from(reserves.0), U256::from(reserves.1))
    } else {
        (U256::from(reserves.1), U256::from(reserves.0))
    };

    let amount_in_with_fee = amount_in * U256::from(997);
    let numerator = amount_in_with_fee * reserve_out;
    let denominator = reserve_in * U256::from(1000) + amount_in_with_fee;
    numerator / denominator
}

fn get_quoter_address(chain_id: ChainId) -> Address {
    match chain_id {
        ChainId::Ethereum(_) => Address::from_str("0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6").unwrap(),
        ChainId::Arbitrum(_) => Address::from_str("0x61fFE014bA17989E743c5F6cB21bF9697530B21e").unwrap(),
        ChainId::Base(_) => Address::from_str("0x3d4e44Eb1374240CE5F1B871ab261CD16335B76a").unwrap(),
        _ => todo!(),
    }
}