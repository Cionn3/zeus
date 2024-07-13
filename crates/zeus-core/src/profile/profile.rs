use super::super::encryption::{Credentials, encrypt_data, decrypt_data};
use super::{ Wallet, WalletBalance, WalletData};
use alloy::core::hex::encode;
use alloy::primitives::Address;
use std::collections::HashMap;
use std::str::FromStr;
use anyhow::anyhow;

const FILENAME: &str = "profile.data";


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
        encrypt_data(FILENAME, data, self.credentials.clone())?;
        Ok(())
    }

    /// Decrypt and load the profile
    pub fn decrypt_and_load(&mut self) -> Result<(), anyhow::Error> {
        let data = decrypt_data(FILENAME, self.credentials.clone())?;
        
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
        if let Err(e) = decrypt_data(FILENAME, credentials.clone()) {
            return Err(anyhow!("Invalid credentials: {}", e));
        }

        let wallet = self.wallets.iter().find(|w| w.name == wallet_name).ok_or_else(|| anyhow!("Wallet not found"))?;
        Ok(wallet.get_key())
    }

    /// Create a new random wallet and add it to the profile
    pub fn new_wallet(&mut self, name: String) -> Result<(), anyhow::Error> {
        // do not allow duplicate names
        if self.wallets.iter().any(|w| w.name == name) {
            return Err(anyhow!("Wallet with name {} already exists", name));
        }
        let wallet = Wallet::new_rng(name);
        self.wallets.push(wallet);
        Ok(())
    }

    /// Import a wallet from a private key
    pub fn import_wallet(&mut self, name: String, balance: HashMap<u64, WalletBalance>, key: String) -> Result<(), anyhow::Error> {
        // do not allow duplicate names
        if self.wallets.iter().any(|w| w.name == name) {
            return Err(anyhow!("Wallet with name {} already exists", name));
        }
        let wallet = Wallet::new_from_key(name, balance, key)?;
        self.wallets.push(wallet);
        Ok(())
    }

    /// Get current wallet
    pub fn current_wallet_name(&self) -> String {
        if let Some(wallet) = &self.current_wallet {
            wallet.name.clone()
        } else {
            "No Wallet Available".to_string()
        }
    }

    /// Truncate the wallet name if its an Ethereum address
    pub fn truncated_name(&self) -> String {
        if let Some(wallet) = &self.current_wallet {
            if let Ok(address) = Address::from_str(&wallet.name) {
                // truncate the address
                return format!("0x{}", &address.to_string()[2..12]);
            }
            wallet.name.clone()
        } else {
            "No Wallet Available".to_string()
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
            let wallet = Wallet::new_from_key(data.name, data.balance, data.key)?;
            wallets.push(wallet);
        }
        Ok(wallets)
    }


}