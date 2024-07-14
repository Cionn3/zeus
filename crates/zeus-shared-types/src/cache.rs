use alloy_primitives::{Address, U256};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use tracing::trace;
use zeus_chain::Currency;
use zeus_core::lazy_static::lazy_static;

lazy_static! {
    pub static ref SHARED_CACHE: Arc<RwLock<SharedCache>> =
        Arc::new(RwLock::new(SharedCache::default()));
}

/// Cached data that can be safely shared across all tasks
///
/// - `erc20_balance` - A map of all token balances for each chain
///
/// - `currencies` - A map of all currencies for each chain
pub struct SharedCache {
    /// The current block number
    pub block: u64,

    /// ERC20 Balance Map
    ///
    /// `Key:` (chain_id, owner, token) -> `Value:` Balance
    pub erc20_balance: HashMap<(u64, Address, Address), U256>,

    /// Eth Balance Map
    ///
    /// `Key:` (chain_id, owner) -> `Value:` (block, balance)
    pub eth_balance: HashMap<(u64, Address), (u64, U256)>,

    /// A Map with all currencies for each chain
    pub currencies: HashMap<u64, Vec<Currency>>,
}

impl SharedCache {
    /// Get the balance of a token for a specific chain_id
    pub fn get_erc20_balance(&self, chain_id: &u64, owner: &Address, token: &Address) -> U256 {
        if let Some(balance) = self.erc20_balance.get(&(*chain_id, *owner, *token)) {
            *balance
        } else {
            trace!("No balance found for token: {:?}", token);
            U256::ZERO
        }
    }

    /// Update the balance of a token for a specific chain_id
    pub fn update_erc20_balance(
        &mut self,
        chain_id: u64,
        owner: Address,
        token: Address,
        balance: U256,
    ) {
        self.erc20_balance.insert((chain_id, owner, token), balance);

        trace!("Updated ERC20 Balance: {:?}", self.erc20_balance);
    }

    /// Get eth balance of a wallet for a specific chain
    pub fn get_eth_balance(&self, chain_id: u64, owner: Address) -> (u64, U256) {
        if let Some(balance) = self.eth_balance.get(&(chain_id, owner)) {
            (balance.0, balance.1)
        } else {
            (0, U256::ZERO)
        }
    }

    /// Update eth balance of a wallet for a specific chain
    pub fn update_eth_balance(&mut self, chain_id: u64, owner: Address, block: u64, balance: U256) {
        self.eth_balance
            .insert((chain_id, owner), (block, balance));
    }

    /// Add a currency
    pub fn add_currency(&mut self, chain_id: u64, currency: Currency) {
        if let Some(currencies) = self.currencies.get_mut(&chain_id) {
            currencies.push(currency);
        } else {
            self.currencies.insert(chain_id, vec![currency]);
        }
    }
}

impl Default for SharedCache {
    fn default() -> Self {
        Self {
            block: 0,
            erc20_balance: HashMap::new(),
            eth_balance: HashMap::new(),
            currencies: HashMap::new(),
        }
    }
}
