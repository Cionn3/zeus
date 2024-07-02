use alloy::{
    primitives::U256,
    providers::{RootProvider, ProviderBuilder},
    pubsub::PubSubFrontend,
    transports::ws::WsConnect
};
use std::sync::Arc;
use std::str::FromStr;
use bigdecimal::BigDecimal;





pub async fn get_client(url: &str) -> Result<Arc<RootProvider<PubSubFrontend>>, anyhow::Error> {
    let client = ProviderBuilder::new().on_ws(WsConnect::new(url)).await?;
    Ok(Arc::new(client))
}


/// Parse from readable units to wei
pub fn parse_wei(amount: &str, decimals: u8) -> Result<U256, anyhow::Error> {
    let amount = BigDecimal::from_str(amount)?;
    let wei_amount = amount * (10_u64).pow(decimals as u32);
    let wei_str = wei_amount.to_string();
    let wei_str = wei_str.split('.').next().unwrap_or_default();
    let wei = U256::from_str(wei_str)?;
    Ok(wei)
}

/// Format the amount from wei to readable units
pub fn format_wei(amount: &str, decimals: u8) -> String {
    let divisor_str = format!("1{:0>width$}", "", width = decimals as usize);
    let divisor = BigDecimal::from_str(&divisor_str).unwrap_or_default();
    let amount = BigDecimal::from_str(&amount).unwrap_or_default();
    let readable = amount / divisor;
    readable.to_string()
}