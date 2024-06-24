use lazy_static::lazy_static;
use std::sync::{ Arc, RwLock };
use std::collections::HashMap;
use alloy::primitives::Bytes;

use crate::defi::erc20::ERC20Token;

lazy_static! {
    pub static ref SHARED_UI_STATE: Arc<RwLock<SharedUiState>> = Arc::new(
        RwLock::new(SharedUiState::default())
    );
    pub static ref SWAP_UI_STATE: Arc<RwLock<SwapUIState>> = Arc::new(
        RwLock::new(SwapUIState::default())
    );
}

/// Error message to show in the UI
#[derive(Clone, Default)]
pub struct ErrorMsg {
    pub on: bool,

    pub msg: String,
}

impl ErrorMsg {
    pub fn new<T>(on: bool, msg: T) -> Self where T: ToString {
        Self {
            on,
            msg: msg.to_string(),
        }
    }
}

/// Info message to show in the UI
#[derive(Clone, Default)]
pub struct InfoMsg {
    pub on: bool,

    pub msg: String,
}

impl InfoMsg {
    pub fn new<T>(on: bool, msg: T) -> Self where T: ToString {
        Self {
            on,
            msg: msg.to_string(),
        }
    }
}

/// Shared State for some GUI components
#[derive(Clone)]
pub struct SharedUiState {
    /// Swap UI on/off
    pub swap_ui_on: bool,

    /// Network settings UI on/off
    pub networks_on: bool,

    /// New/Import Wallet UI on/off
    ///
    /// (on/off, "New"/"Import Wallet")
    pub wallet_popup: (bool, &'static str),

    /// Export wallet Key UI on/off
    pub export_key_ui: bool,

    /// Exported key window on/off
    pub exported_key_window: (bool, String),

    /// TxSettings popup on/off
    pub tx_settings_on: bool,

    /// Error message to show in the UI
    pub err_msg: ErrorMsg,

    /// Info message to show in the UI
    pub info_msg: InfoMsg,
}

impl Default for SharedUiState {
    fn default() -> Self {
        Self {
            swap_ui_on: true,
            networks_on: false,
            wallet_popup: (false, "New"),
            export_key_ui: false,
            exported_key_window: (false, String::new()),
            tx_settings_on: false,
            err_msg: ErrorMsg::default(),
            info_msg: InfoMsg::default(),
        }
    }
}

#[derive(Clone, Default)]
pub struct QuoteResult {

    /// Block Number
    pub block_number: u64,

    pub input_token: SelectedToken,

    pub output_token: SelectedToken,

    /// USD worth of the input token
    pub input_token_usd_worth: String,

    /// USD worth of the output token
    pub output_token_usd_worth: String,

    /// The price impact of the swap
    pub price_impact: String,

    /// Selected slippage
    pub slippage: String,

    /// The real amount of tokens we will receive, after considering the pool fee and token tax if any
    pub real_amount: String,

    /// Minimum amount we may receive depending on the slippage
    pub minimum_received: String,

    /// Token Tax (If any)
    pub token_tax: String,

    /// Pool Fee
    pub pool_fee: String,

    /// Gas Cost of the swap in USD
    pub gas_cost: String,

    /// Call Data to be used for the transaction
    pub data: Bytes,

}

impl QuoteResult {
    /// Get Output token amount in readable format
    pub fn output_token_amount(&self) -> String {
        format!("{} {}", self.output_token.token.readable(self.real_amount.clone()), self.output_token.token.symbol)
    }

    /// Get Minimum received amount in readable format
    pub fn minimum_received_amount(&self) -> String {
        format!("{} {}", self.output_token.token.readable(self.minimum_received.clone()), self.output_token.token.symbol)
    }

}

/// A token that its currently selected in the SwapUI
#[derive(Clone, PartialEq, Default)]
pub struct SelectedToken {
    pub token: ERC20Token,

    /// The amount of tokens to swap
    pub amount_to_swap: String,

    /// The balance the owner has for this token
    pub balance: String,
}

/// The state of the SwapUI
pub struct SwapUIState {
    /// Latest Block Number
    pub block: u64,

    /// Status Of The `Input Token` Window List
    pub input_token_list_on: bool,

    /// Status Of The `Output Token` Window List
    pub output_token_list_on: bool,

    /// The input token
    pub input_token: SelectedToken,

    /// The output token
    pub output_token: SelectedToken,

    /// The search query for a token
    pub search_token: String,

    /// A HashMap that holds a list of [ERC20Token] with their corrsponing `chain_id` as key
    pub tokens: HashMap<u64, Vec<ERC20Token>>,

    pub quote_result: QuoteResult,
}

impl SwapUIState {
    /// Get the input or output token by an id
    pub fn get_token(&self, id: &str) -> SelectedToken {
        match id {
            "input" => self.input_token.clone(),
            "output" => self.output_token.clone(),
            _ => SelectedToken::eth_default_input(),
        }
    }

    /// Replace the input or output token by an id
    pub fn replace_token(&mut self, id: &str, token: SelectedToken) {
        match id {
            "input" => {
                self.input_token = token;
            }
            "output" => {
                self.output_token = token;
            }
            _ => {}
        }
    }

    /// Update the balance of a [SelectedToken]
    pub fn update_balance(&mut self, id: &str, balance: String) {
        match id {
            "input" => {
                self.input_token.balance = balance;
            }
            "output" => {
                self.output_token.balance = balance;
            }
            _ => {}
        }
    }

    /// Get which list is on or off by an id
    ///
    /// `id` -> "input" or "output" token
    pub fn get_token_list_status(&self, id: &str) -> bool {
        match id {
            "input" => self.input_token_list_on,
            "output" => self.output_token_list_on,
            _ => false,
        }
    }

    /// Close or Open the [token_list_window] by an id
    ///
    /// `id` -> "input" or "output" token
    ///
    /// `on` -> true or false
    pub fn update_token_list_status(&mut self, id: &str, on: bool) {
        match id {
            "input" => {
                self.input_token_list_on = on;
            }
            "output" => {
                self.output_token_list_on = on;
            }
            _ => {}
        }
    }

    /// Update input_token based on the selected chain id
    pub fn default_input(&mut self, id: u64) {
        match id {
            1 => {
                self.input_token = SelectedToken::eth_default_input();
            }
            56 => {
                self.input_token = SelectedToken::bsc_default_input();
            }
            8453 => {
                self.input_token = SelectedToken::base_default_input();
            }
            42161 => {
                self.input_token = SelectedToken::arbitrum_default_input();
            }
            _ => {}
        }
    }

    /// Update output_token based on the selected chain id
    pub fn default_output(&mut self, id: u64) {
        match id {
            1 => {
                self.output_token = SelectedToken::eth_default_output();
            }
            56 => {
                self.output_token = SelectedToken::bsc_default_output();
            }
            8453 => {
                self.output_token = SelectedToken::base_default_output();
            }
            42161 => {
                self.output_token = SelectedToken::arbitrum_default_output();
            }
            _ => {}
        }
    }
}

impl Default for SwapUIState {
    fn default() -> Self {
        Self {
            block: 0,
            input_token_list_on: false,
            output_token_list_on: false,
            input_token: SelectedToken::eth_default_input(),
            output_token: SelectedToken::eth_default_output(),
            search_token: String::new(),
            tokens: HashMap::new(),
            quote_result: QuoteResult::default(),
        }
    }
}

impl SelectedToken {
    pub fn new(token: ERC20Token) -> Self {
        Self {
            token,
            amount_to_swap: String::new(),
            balance: "0".to_string(),
        }
    }

    pub fn eth_default_input() -> Self {
        Self {
            token: ERC20Token::eth_default_input(),
            amount_to_swap: String::new(),
            balance: "0".to_string(),
        }
    }

    pub fn eth_default_output() -> Self {
        Self {
            token: ERC20Token::eth_default_output(),
            amount_to_swap: String::new(),
            balance: "0".to_string(),
        }
    }

    pub fn bsc_default_input() -> Self {
        Self {
            token: ERC20Token::bsc_default_input(),
            amount_to_swap: String::new(),
            balance: "0".to_string(),
        }
    }

    pub fn bsc_default_output() -> Self {
        Self {
            token: ERC20Token::bsc_default_output(),
            amount_to_swap: String::new(),
            balance: "0".to_string(),
        }
    }

    pub fn base_default_input() -> Self {
        Self {
            token: ERC20Token::base_default_input(),
            amount_to_swap: String::new(),
            balance: "0".to_string(),
        }
    }

    pub fn base_default_output() -> Self {
        Self {
            token: ERC20Token::base_default_output(),
            amount_to_swap: String::new(),
            balance: "0".to_string(),
        }
    }

    pub fn arbitrum_default_input() -> Self {
        Self {
            token: ERC20Token::arbitrum_default_input(),
            amount_to_swap: String::new(),
            balance: "0".to_string(),
        }
    }

    pub fn arbitrum_default_output() -> Self {
        Self {
            token: ERC20Token::arbitrum_default_output(),
            amount_to_swap: String::new(),
            balance: "0".to_string(),
        }
    }
}
