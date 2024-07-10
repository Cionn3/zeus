use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::str::FromStr;

use alloy::{
    primitives::{U256, hex::encode},
    signers::{local::{LocalSigner, PrivateKeySigner}, k256::ecdsa::SigningKey},
};


/// Eth balance at a specific block
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WalletBalance {
    pub balance: U256,
    pub block: u64
}

impl Default for WalletBalance {
    fn default() -> Self {
        Self {
            balance: U256::ZERO,
            block: 0,
        }
    }
}



/// Helper struct to serialize wallets that are about to be encrypted in a `profile.data` file
#[derive(Clone, Serialize, Deserialize)]
pub struct WalletData {
    pub name: String,
    pub balance: HashMap<u64, WalletBalance>,
    pub key: String,
}

/// Represents a wallet
/// 
/// - `name` - The given name of the wallet `(if empty, the address is used)`
/// - `balance` - The `Eth` Balance of the wallet for a specific chain
/// - `key` - The key of the wallet
#[derive(Debug, Clone, PartialEq)]
pub struct Wallet {
    /// The given name of the wallet
    pub name: String,

    /// The Eth Balance of the wallet for a specific chain
    pub balance: HashMap<u64, WalletBalance>,
    
    /// The key of the wallet
    pub key: LocalSigner<SigningKey>,
}

impl Wallet {

    /// Get the wallets eth balance for a specific chain
    pub fn get_balance(&self, id: u64) -> U256 {
        self.balance.get(&id).map_or(U256::ZERO, |b| b.balance)
    }

    /// Check if the wallets's balance its outdated
    pub fn balance_outdated(&self, chain_id: u64, latest_block: u64) -> bool {
        if let Some(wallet_balance) = self.balance.get(&chain_id) {
            wallet_balance.block < latest_block 
        } else {
            true
        }
    }

    /// Update eth balance for a specific chain and block
    pub fn update_balance(&mut self, id: u64, balance: U256, block: u64) {
        let balance = WalletBalance { balance, block };
        self.balance.insert(id, balance);
        // remove old balances for the same chain
        self.balance.retain(|&id, b| id != id || b.block == block);
    }

    /// Get wallet's key in string format
    pub fn get_key(&self) -> String {
        let key_vec = self.key.to_bytes().to_vec();
        encode(key_vec)
    }

    /// Create a new wallet with a random private key
    pub fn new_rng(name: String) -> Self {
        let key = PrivateKeySigner::random();

        let name = if name.is_empty() {
            key.address().to_string()
        } else {
            name
        };

        Self {
            name,
            balance: HashMap::new(),
            key,
        }
    }

    /// Create a new wallet from a given private key
    pub fn new_from_key(name: String, key_str: String) -> Result<Self, anyhow::Error> {
        let key = PrivateKeySigner::from_str(&key_str)?;  

        let name = if name.is_empty() {
            key.address().to_string()
        } else {
            name
        };

        Ok(
        Self {
            name,
            balance: HashMap::new(),
            key,
        })
    }


}