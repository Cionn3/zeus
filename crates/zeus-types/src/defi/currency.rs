use super::erc20::ERC20Token;

/// Represents a Currency, this can be a native currency to its chain (eg ETH, BNB) or any ERC20 token
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Currency {
    Native(NativeCurrency),
    ERC20(ERC20Token),
}

impl Currency {
    /// Creates a new default native currency based on the chain_id
    pub fn new_native(chain_id: u64) -> Self {
        match chain_id {
            1 =>
                Self::Native(
                    NativeCurrency::new(chain_id, "ETH".to_string(), "Ether".to_string(), 18, None)
                ),
            56 =>
                Self::Native(
                    NativeCurrency::new(chain_id, "BNB".to_string(), "Binance Coin".to_string(), 18, None)
                ),
            8453 =>
                Self::Native(
                    NativeCurrency::new(chain_id, "ETH".to_string(), "Ether".to_string(), 18, None)
                ),
            42161 =>
                Self::Native(
                    NativeCurrency::new(chain_id, "ETH".to_string(), "Ether".to_string(), 18, None)
                ),
            // * This should not happen!
            _ =>
                Self::Native(
                    NativeCurrency::new(chain_id, "ETH".to_string(), "Ether".to_string(), 18, None)
                ),
        }
    }

    /// Create a new from an already existing native currency
    pub fn new_from_native(native: NativeCurrency) -> Self {
        Self::Native(native)
    }

    /// Creates a new ERC20 token
    pub fn new_erc20(erc20: ERC20Token) -> Self {
        Self::ERC20(erc20)
    }

    /// Created a new Default ERC20 token based on the chain_id
    pub fn default_erc20(chain_id: u64) -> Self {
        match chain_id {
            1 => Self::ERC20(ERC20Token::eth_default_input()),
            56 => Self::ERC20(ERC20Token::bsc_default_input()),
            8453 => Self::ERC20(ERC20Token::base_default_input()),
            42161 => Self::ERC20(ERC20Token::arbitrum_default_input()),
            // * This should not happen!
            _ => Self::ERC20(ERC20Token::eth_default_input()),
        }
    }

    /// Returns if the currency is native
    pub fn is_native(&self) -> bool {
        matches!(self, Self::Native(_))
    }

    /// Get currency symbol
    pub fn symbol(&self) -> String {
        match self {
            Self::Native(native) => native.symbol.clone(),
            Self::ERC20(erc20) => erc20.symbol.clone(),
        }
    }

}

impl Default for Currency {
    fn default() -> Self {
        Self::Native(NativeCurrency::default())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeCurrency {
    pub chain_id: u64,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub icon: Option<Vec<u8>>,
}

impl NativeCurrency {
    pub fn new(chain_id: u64, symbol: String, name: String, decimals: u8, icon: Option<Vec<u8>>) -> Self {
        Self {
            chain_id,
            symbol,
            name,
            decimals,
            icon,
        }
    }

    pub fn default() -> Self {
        Self {
            chain_id: 1,
            symbol: "ETH".to_string(),
            name: "Ether".to_string(),
            decimals: 18,
            icon: None,
        }
    }
}
