use std::str::FromStr;
use alloy::primitives::U256;
use alloy::signers::wallet::{LocalWallet, Wallet as AlloyWallet};
use alloy::signers::k256::ecdsa::SigningKey;
use alloy::core::hex::encode;
use serde::{Serialize, Deserialize};
use sha2::{ Sha256, digest::Digest };
use password_hash::SaltString;
use anyhow::anyhow;


pub mod encryption;

pub const FILENAME: &'static str = "profile.data";


/// The credentials needed to encrypt and decrypt a `profile.data` file
#[derive(Clone, Debug, PartialEq)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub confrim_password: String,
}

impl Default for Credentials {
    fn default() -> Self {
        Self {
            username: Default::default(),
            password: Default::default(),
            confrim_password: Default::default(),
        }
    }
}

impl Credentials {
    /// Salt for Argon2
    fn generate_saltstring(&self) -> SaltString {
        let salt_array = Sha256::digest(self.username.as_bytes());
        let salt = salt_array.to_vec();
        let salt = String::from(
            salt
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>()
        );
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


/// Helper struct to store wallets are about to be saved in a `profile.data` file
#[derive(Clone, Serialize, Deserialize)]
pub struct WalletData {
    pub name: String,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Wallet {
    /// The given name of the wallet
    pub name: String,

    /// The Balance of the wallet
    pub balance: U256,
    
    /// The key of the wallet
    pub key: AlloyWallet<SigningKey>,
}

impl Wallet {

    /// Get wallet's key in string format
    pub fn get_key(&self) -> String {
        let key_vec = self.key.signer().to_bytes().to_vec();
        encode(&key_vec)
    }

    /// Create a new wallet with a random private key
    pub fn new_rng(name: String) -> Self {
        let key = LocalWallet::random();

        let name = if name.is_empty() {
            key.address().to_string()
        } else {
            name
        };

        Self {
            name,
            balance: U256::ZERO,
            key,
        }
    }

    /// Create a new wallet from a given private key
    pub fn new_from_key(name: String, key_str: String) -> Result<Self, anyhow::Error> {
        let key = LocalWallet::from_str(&key_str)?;  

        let name = if name.is_empty() {
            key.address().to_string()
        } else {
            name
        };

        Ok(
        Self {
            name,
            balance: U256::ZERO,
            key,
        })
    }


}

/// Information for a given `profile.data` file
/// 
/// Only `wallets` remain and encrypted locally
/// 
/// If the user forgots the username or password, the contents of this file are lost forever
#[derive(Debug, Clone, PartialEq)]
pub struct Profile {

    /// Credentials of the profile
    pub credentials: Credentials,

    /// The wallets of the profile
    pub wallets: Vec<Wallet>,

    /// The current selected wallet
    pub current_wallet: Option<Wallet>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            credentials: Credentials::default(),
            wallets: Vec::new(),
            current_wallet: None,
        }
    }

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
            let key_vec = wallet.key.signer().to_bytes().to_vec();
            let key = encode(&key_vec);
            let data = WalletData {
                name: wallet.name.clone(),
                key,
            };
            wallet_data.push(data);
        }
        serde_json::to_string(&wallet_data).map_err(|e| anyhow::Error::new(e))
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