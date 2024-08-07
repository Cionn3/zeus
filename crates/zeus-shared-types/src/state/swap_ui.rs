use std::sync::{ Arc, RwLock };

use zeus_chain::{
    alloy::primitives::{ Bytes, U256, Address },
    defi_types::currency::{ Currency, NativeCurrency, erc20::ERC20Token },
};
use crate::cache::{SHARED_CACHE, SharedCache};
use zeus_core::lazy_static::lazy_static;

lazy_static! {

    /// The State Of the Swap UI
    /// 
    /// This can be safely shared across all tasks
    pub static ref SWAP_UI_STATE: Arc<RwLock<SwapUIState>> = Arc::new(
        RwLock::new(SwapUIState::default())
    );
}

#[derive(Clone, Default)]
pub struct QuoteResult {
    /// Block Number
    pub block_number: u64,

    pub input_token: SelectedCurrency,

    pub output_token: SelectedCurrency,

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
        "TODO".to_string()
    }

    /// Get Minimum received amount in readable format
    pub fn minimum_received_amount(&self) -> String {
        "TODO".to_string()
    }
}

/// A currency that its currently selected in a UI
#[derive(Debug, Clone, PartialEq)]
pub struct SelectedCurrency {
    pub currency: Currency,

    /// The amount of currency to swap
    pub amount_to_swap: String,
}

/// The state of the SwapUI
pub struct SwapUIState {
    /// Latest Block Number
    pub block: u64,

    /// Status of the `Currency In` Window selection list
    pub currency_in_list_on: bool,

    /// Status of the `Currency Out` Window selection list
    pub currency_out_list_on: bool,

    /// Currency to swap from
    pub currency_in: SelectedCurrency,

    /// Currency to swap to
    pub currency_out: SelectedCurrency,

    /// The search query for a currency
    pub search_currency: String,

    pub shared_cache: Arc<RwLock<SharedCache>>,

    pub quote_result: QuoteResult,
}

impl SwapUIState {

    /// Get the balance of a token for a specific chain_id
    pub fn get_erc20_balance(&self, chain_id: &u64, owner: &Address, token: &Address) -> U256 {
        self.shared_cache.read().unwrap().get_erc20_balance(chain_id, owner, token)
    }

    /// Update the balance of a token for a specific chain_id
    pub fn update_erc20_balance(&mut self, chain_id: u64, owner: Address, token: Address, balance: U256) {
        self.shared_cache.write().unwrap().update_erc20_balance(chain_id, owner, token, balance);
    }

    /// Get the balance of a [SelectedCurrency]
    pub fn get_balance(&self, chain_id: &u64, owner: &Address, currency: &SelectedCurrency) -> U256 {
        if currency.currency.is_native() {
            self.shared_cache.read().unwrap().get_eth_balance(*chain_id, *owner).1
        } else {
            let token = currency.get_erc20().unwrap();
            self.get_erc20_balance(chain_id, owner, &token.address)
        }
    }

    /// Get the input or output selected currency by an id
    pub fn get_selected_currency(&self, id: &str) -> SelectedCurrency {
        match id {
            "input" => self.currency_in.clone(),
            "output" => self.currency_out.clone(),
            // * This should not happen
            _ => SelectedCurrency::default(),
        }
    }

    /// Replace the input or output currency by an id
    pub fn replace_currency(&mut self, id: &str, currency: SelectedCurrency) {
        match id {
            "input" => {
                self.currency_in = currency;
            }
            "output" => {
                self.currency_out = currency;
            }
            _ => {}
        }
    }


    /// Get which window selection list is on or off by an id
    ///
    /// `id` -> "input" or "output" currency
    pub fn get_currency_list_status(&self, id: &str) -> bool {
        match id {
            "input" => self.currency_in_list_on,
            "output" => self.currency_out_list_on,
            _ => false,
        }
    }

    /// Close or Open a currency selection window by an id
    ///
    /// `id` -> "input" or "output" currency
    ///
    /// `on` -> true or false
    pub fn update_token_list_status(&mut self, id: &str, on: bool) {
        match id {
            "input" => {
                self.currency_in_list_on = on;
            }
            "output" => {
                self.currency_out_list_on = on;
            }
            _ => {}
        }
    }

    /// Give a default input currency based on the selected chain id
    pub fn default_input(&mut self, id: u64) {
        self.currency_in = SelectedCurrency::default_input(id);
    }

    /// Give a default output currency based on the selected chain id
    pub fn default_output(&mut self, id: u64) {
        self.currency_out = SelectedCurrency::default_output(id);
    }
}

impl Default for SwapUIState {
    fn default() -> Self {
        Self {
            block: 0,
            currency_in_list_on: false,
            currency_out_list_on: false,
            currency_in: SelectedCurrency::default_input(1),
            currency_out: SelectedCurrency::default_output(1),
            search_currency: String::new(),
            shared_cache: SHARED_CACHE.clone(),
            quote_result: QuoteResult::default(),
        }
    }
}

impl SelectedCurrency {
    /// Create a new selected currency from an ERC20Token
    pub fn new_from_erc(token: ERC20Token) -> Self {
        Self {
            currency: Currency::new_erc20(token),
            amount_to_swap: String::new(),
        }
    }

    /// Create a new selected currency from a native currency
    pub fn new_from_native(currency: NativeCurrency) -> Self {
        Self {
            currency: Currency::new_from_native(currency),
            amount_to_swap: String::new(),
        }
    }

    /// Create a default input currency based on the chain_id
    pub fn default_input(id: u64) -> Self {
        Self {
            currency: Currency::new_native(id),
            amount_to_swap: String::new(),
        }
    }

    /// Creates a default output currency based on the chain_id
    pub fn default_output(id: u64) -> Self {
        Self {
            currency: Currency::default_erc20(id),
            amount_to_swap: String::new(),        }
    }

    pub fn is_native(&self) -> bool {
        self.currency.is_native()
    }

    /// Gets the erc20 inside the selected currency
    pub fn get_erc20(&self) -> Option<ERC20Token> {
        match &self.currency {
            Currency::ERC20(erc20) => Some(erc20.clone()),
            _ => None,
        }
    }

    /// Gets the decimals of the selected currency
    pub fn decimals(&self) -> u8 {
        match &self.currency {
            Currency::Native(currency) => currency.decimals,
            Currency::ERC20(erc20) => erc20.decimals,
        }
    }

    pub fn name(&self) -> String {
        match &self.currency {
            Currency::Native(currency) => currency.name.clone(),
            Currency::ERC20(erc20) => erc20.name.clone(),
        }
    
    }
}

impl Default for SelectedCurrency {
    fn default() -> Self {
        Self {
            currency: Currency::default(),
            amount_to_swap: String::new(),
        }
    }
}
