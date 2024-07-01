use alloy::primitives::U256;
use bigdecimal::BigDecimal;
use std::str::FromStr;

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
