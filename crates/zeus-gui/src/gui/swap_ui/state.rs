use std::collections::HashMap;
use zeus_defi::erc20::ERC20Token;

use lazy_static::lazy_static;
use std::sync::{Arc, RwLock};

lazy_static! {
    pub static ref SWAP_UI_STATE: Arc<RwLock<SwapUIState>> = Arc::new(RwLock::new(SwapUIState::default()));
}

/// A token that its currently selected in the SwapUI
pub struct SelectedToken {

    pub token: ERC20Token,

    /// The amount of tokens to swap
    pub amount_to_swap: String,

    /// The balance the owner has for this token
    pub balance: String,
}

/// The state of the SwapUI
pub struct SwapUIState {

    /// Switch the UI on or off
    pub on: bool,

    /// Close or Open the `Input Token` Window List
    pub input_token_list_on: bool,

    /// Close or Open the `Output Token` Window List
    pub output_token_list_on: bool,

    /// The input token
    pub input_token: SelectedToken,

    /// The output token
    pub output_token: SelectedToken,

    /// The search query for a token
    pub search_token: String,

    /// A HashMap that holds a list of [ERC20Token] with their corrsponing `chain_id` as key
    pub tokens: HashMap<u64, Vec<ERC20Token>>,
}

impl Default for SwapUIState {
    fn default() -> Self {
        Self {
            on: true,
            input_token_list_on: false,
            output_token_list_on: false,
            input_token: SelectedToken::eth_default_input(),
            output_token: SelectedToken::eth_default_output(),
            search_token: String::new(),
            tokens: HashMap::new(),
        }
    }
}


impl SelectedToken {
    pub fn eth_default_input() -> Self {
        Self {
            token: ERC20Token::eth_default_input(),
            amount_to_swap: String::new(),
            balance: String::new(),
        }
    }

    pub fn eth_default_output() -> Self {
        Self {
            token: ERC20Token::eth_default_output(),
            amount_to_swap: String::new(),
            balance: String::new(),
        }
    }

    pub fn bsc_default_input() -> Self {
        Self {
            token: ERC20Token::bsc_default_input(),
            amount_to_swap: String::new(),
            balance: String::new(),
        }
    }

    pub fn bsc_default_output() -> Self {
        Self {
            token: ERC20Token::bsc_default_output(),
            amount_to_swap: String::new(),
            balance: String::new(),
        }
    }

    pub fn base_default_input() -> Self {
        Self {
            token: ERC20Token::base_default_input(),
            amount_to_swap: String::new(),
            balance: String::new(),
        }
    }

    pub fn base_default_output() -> Self {
        Self {
            token: ERC20Token::base_default_output(),
            amount_to_swap: String::new(),
            balance: String::new(),
        }
    }

    pub fn arbitrum_default_input() -> Self {
        Self {
            token: ERC20Token::arbitrum_default_input(),
            amount_to_swap: String::new(),
            balance: String::new(),
        }
    }

    pub fn arbitrum_default_output() -> Self {
        Self {
            token: ERC20Token::arbitrum_default_output(),
            amount_to_swap: String::new(),
            balance: String::new(),
        }
    }
}