use alloy_primitives::{Address, U256};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use zeus_chain::Currency;
use zeus_core::lazy_static::lazy_static;
use tracing::trace;

lazy_static! {
    pub static ref SHARED_CACHE: Arc<RwLock<SharedCache>> =
        Arc::new(RwLock::new(SharedCache::default()));
}

/// Data that we cache in memory that can be safely shared across all tasks
///
/// - `erc20_balance` - A map of all token balances for each chain
///
/// - `currencies` - A map of all currencies for each chain
pub struct SharedCache {
    /// The current block number
    pub block: u64,

    erc20_balance: HashMap<u64, (u64, Address, U256)>,

    pub currencies: HashMap<u64, Vec<Currency>>,
}

impl SharedCache {
    /// Get the balance of an ERC20 token
    pub fn get_erc20_balance(&self, chain: u64, token: &Address) -> U256 {
        match self.erc20_balance.get(&chain) {
            Some((_, addr, balance)) if addr == token => balance.clone(),
            _ => U256::from(0),
        }
    }

    /// Update the balance of a token for a specific chain_id
    pub fn update_erc20_balance(&mut self, chain_id: u64, token: Address, balance: U256) {
        self.erc20_balance
            .insert(chain_id, (self.block, token, balance));

        // remove old balances < block for the same chain and token only
        self.erc20_balance
            .retain(|_, (block, addr, _)| *block >= self.block || *addr != token);
        trace!("Updated ERC20 Balance: {:?}", self.erc20_balance);
    }

    /// Add a currency
    pub fn add_currency(&mut self, chain_id: u64, currency: Currency) {
        self.currencies
            .entry(chain_id)
            .or_insert_with(Vec::new)
            .push(currency);
    }
}

impl Default for SharedCache {
    fn default() -> Self {
        Self {
            block: 0,
            erc20_balance: HashMap::new(),
            currencies: HashMap::new(),
        }
    }
}
