use std::str::FromStr;
use std::collections::HashMap;
use alloy::primitives::U256;
use alloy::signers::local::{PrivateKeySigner, LocalSigner};
use alloy::signers::k256::ecdsa::SigningKey;
use alloy::core::hex::encode;
use serde::{Serialize, Deserialize};
use sha2::{ Sha256, digest::Digest };
use password_hash::SaltString;
use anyhow::anyhow;


pub mod encryption;

pub const FILENAME: &str = "profile.data";


/// The credentials needed to encrypt and decrypt a `profile.data` file
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub confrim_password: String,
}


impl Credentials {
    /// Salt for Argon2
    fn generate_saltstring(&self) -> SaltString {
        let salt_array = Sha256::digest(self.username.as_bytes());
        let salt = salt_array.to_vec();
        let salt = salt.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        SaltString::from_b64(&salt).unwrap()
    }

    fn is_valid(&self) -> Result<(), anyhow::Error> {
        if self.username.is_empty() || self.password.is_empty() || self.confrim_password.is_empty() {
            return Err(anyhow!("Username and Password must be provided"));
        }

        if self.password != self.confrim_password {
            return Err(anyhow!("Passwords do not match"));
        }

        Ok(())
    }
}

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

/// Helper struct to store wallets are about to be saved in a `profile.data` file
#[derive(Clone, Serialize, Deserialize)]
pub struct WalletData {
    pub name: String,
    pub balance: HashMap<u64, WalletBalance>,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Wallet {
    /// The given name of the wallet
    pub name: String,

    /// The Balance of the wallet for a specific chain
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
    pub fn should_update(&self, chain_id: u64, latest_block: u64) -> bool {
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

/// Information for a given `profile.data` file
/// 
/// Only `wallets` remain and encrypted locally
/// 
/// If the user forgots the username or password, the contents of this file are lost forever
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Profile {

    /// Credentials of the profile
    pub credentials: Credentials,

    /// The wallets of the profile
    pub wallets: Vec<Wallet>,

    /// The current selected wallet
    pub current_wallet: Option<Wallet>,
}


impl Profile {

    /// Encrypt and save the wallets of the profile
    pub fn encrypt_and_save(&self) -> Result<(), anyhow::Error> {
        let data = self.serialize_to_json()?.as_bytes().to_vec();
        encryption::encrypt_data(FILENAME, data, self.credentials.clone())?;
        Ok(())
    }

    /// Decrypt and load the profile
    pub fn decrypt_and_load(&mut self) -> Result<(), anyhow::Error> {
        let data = encryption::decrypt_data(FILENAME, self.credentials.clone())?;
        
        let wallets = Profile::deserialize_from_json(data)?;
        self.wallets = wallets;

        // if there is at least 1 wallet available, set the current wallet to the first one
        if !self.wallets.is_empty() {
            self.current_wallet = Some(self.wallets[0].clone());
        }

        Ok(())
    }

    /// Confirm again the credentials and export the givens wallet key
    pub fn export_wallet(&self, wallet_name: String, credentials: Credentials) -> Result<String, anyhow::Error> {
        if let Err(e) = encryption::decrypt_data(FILENAME, credentials.clone()) {
            return Err(anyhow!("Invalid credentials: {}", e));
        }

        let wallet = self.wallets.iter().find(|w| w.name == wallet_name).ok_or_else(|| anyhow!("Wallet not found"))?;
        Ok(wallet.get_key())
    }

    /// Create a new random wallet and add it to the profile
    pub fn new_wallet(&mut self, name: String) {
        let wallet = Wallet::new_rng(name);
        self.wallets.push(wallet);
    }

    /// Import a wallet from a private key
    pub fn import_wallet(&mut self, name: String, key: String) -> Result<(), anyhow::Error> {
        let wallet = Wallet::new_from_key(name, key)?;
        self.wallets.push(wallet);
        Ok(())
    }

    /// Add a wallet to the profile
    pub fn add_wallet(&mut self, wallet: Wallet) {
        self.wallets.push(wallet);
    }

    /// Get current wallet
    pub fn current_wallet_name(&self) -> String {
        if let Some(wallet) = &self.current_wallet {
            wallet.name.clone()
        } else {
            "No Wallet".to_string()
        }
    }


    /// Convert all the wallets keys with their names to Json string format
    pub fn serialize_to_json(&self) -> Result<String, anyhow::Error> {
        let mut wallet_data = Vec::new();
        for wallet in self.wallets.iter() {
            let key_vec = wallet.key.to_bytes().to_vec();
            let key = encode(&key_vec);
            let data = WalletData {
                name: wallet.name.clone(),
                balance: wallet.balance.clone(),
                key,
            };
            wallet_data.push(data);
        }
        Ok(serde_json::to_string(&wallet_data)?)
    }
    
    /// Restore the wallets
    pub fn deserialize_from_json(data: Vec<u8>) -> Result<Vec<Wallet>, anyhow::Error> {
        let wallet_data = serde_json::from_slice::<Vec<WalletData>>(&data)?;
        let mut wallets = Vec::new();
        for data in wallet_data {
            let wallet = Wallet::new_from_key(data.name, data.key)?;
            wallets.push(wallet);
        }
        Ok(wallets)
    }


}