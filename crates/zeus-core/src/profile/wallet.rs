use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

use alloy::{
    primitives::{hex::encode, U256},
    signers::{
        k256::ecdsa::SigningKey,
        local::{LocalSigner, PrivateKeySigner},
    },
};

/// Eth balance at a specific block
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WalletBalance {
    pub balance: U256,
    pub block: u64,
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
    pub fn new_from_key(
        name: String,
        balance: HashMap<u64, WalletBalance>,
        key_str: String,
    ) -> Result<Self, anyhow::Error> {
        let key = PrivateKeySigner::from_str(&key_str)?;

        let name = if name.is_empty() {
            key.address().to_string()
        } else {
            name
        };

        Ok(Self { name, balance, key })
    }

    /// Truncate the wallet name if its an Ethereum address
    pub fn truncated_name(&self) -> String {
        if self.name.len() == 42 {
            format!("{}...{}", &self.name[..6], &self.name[38..])
        } else {
            self.name.clone()
        }
    }
}
