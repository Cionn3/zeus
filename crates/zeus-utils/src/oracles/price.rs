








#[derive(Clone)]
pub struct PriceOracle {
    /// Weth price in USD
    weth_usdc: U256,

    chain_id: ChainId,
}