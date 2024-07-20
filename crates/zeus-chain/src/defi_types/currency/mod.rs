

pub mod erc20;
use self::erc20::ERC20Token;

/// Represents a Currency, this can be a [NativeCurrency] to its chain (eg ETH, BNB) or any [ERC20Token]
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
                    NativeCurrency::default()
                ),
            56 =>
                Self::Native(
                    NativeCurrency::default_for_chain(&56)
                ),
            8453 =>
                Self::Native(
                    NativeCurrency::default_for_chain(&8453)
                ),
            42161 =>
                Self::Native(
                    NativeCurrency::default_for_chain(&42161)
                ),
            // * This should not happen!
            _ =>
                Self::Native(
                    NativeCurrency::default()
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

    /// Get the ERC20 Token
    pub fn erc20(&self) -> Option<&ERC20Token> {
        match self {
            Self::ERC20(erc20) => Some(erc20),
            _ => None,
        }
    }

    /// Get currency symbol
    pub fn symbol(&self) -> String {
        match self {
            Self::Native(native) => native.symbol.clone(),
            Self::ERC20(erc20) => erc20.symbol.clone(),
        }
    }

    /// Get currency name
    pub fn name(&self) -> String {
        match self {
            Self::Native(native) => native.name.clone(),
            Self::ERC20(erc20) => erc20.name.clone(),
        }
    }

    /// Returns the decimals
    pub fn decimals(&self) -> u8 {
        match self {
            Self::Native(native) => native.decimals,
            Self::ERC20(erc20) => erc20.decimals,
        }
    }

}

impl Default for Currency {
    fn default() -> Self {
        Self::Native(NativeCurrency::default())
    }
}

/// Represents a Native Currency to its chain
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
            name: "Ethereum".to_string(),
            decimals: 18,
            icon: None,
        }
    }

    /// A Default Native Currency for a chain id
    pub fn default_for_chain(id: &u64) -> Self {
        match id {
            1 => Self::default(),
            56 => Self::new(56, "BNB".to_string(), "Binance Coin".to_string(), 18, None),
            8453 => Self::new(8453, "ETH".to_string(), "Ethereum".to_string(), 18, None),
            42161 => Self::new(42161, "ETH".to_string(), "Ethereum".to_string(), 18, None),
            // * This should not happen!
            _ => Self::default(),
        }
    }
}
