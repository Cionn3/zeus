use std::{path::Path, str::FromStr};
use std::sync::Arc;

use alloy::{
    providers::RootProvider,
    pubsub::PubSubFrontend,
    primitives::U256
};


use crate::{profile::{Credentials, Profile}, ChainId, Rpc};

/// Transaction settings
#[derive(Clone)]
pub struct TxSettings {
    pub priority_fee: String,
    pub slippage: String,
    pub mev_protect: bool,
}

impl TxSettings {

    /// Parse a string to gwei
    pub fn parse_gwei(&self) -> U256 {
        let amount = U256::from_str(&self.priority_fee).unwrap_or(U256::from(3));
        amount * U256::from(10).pow(U256::from(9))
    }

    /// Parse a string to f32
    pub fn parse_slippage(&self) -> f32 {
        self.slippage.parse().unwrap_or(0.5)
    }
}

impl Default for TxSettings {
    fn default() -> Self {
        Self {
            priority_fee: String::from("3"),
            slippage: String::from("0.5"),
            mev_protect: true,
        }
    }
}

/// Main data and settings loaded by the app
#[derive(Clone)]
pub struct AppData {

    /// The current ws client connected to
    pub ws_client: Option<Arc<RootProvider<PubSubFrontend>>>,

    /// The current selected chain id
    pub chain_id: ChainId,

    /// All supported networks
    pub networks: Vec<(ChainId, & 'static str)>,

    /// The current saved RPC endpoints
    pub rpc: Vec<Rpc>,

    /// The current profile
    pub profile: Profile,

    /// Tx settings
    pub tx_settings: TxSettings,

    /// Are we logged in?
    pub logged_in: bool,

    /// New profile screen on/off
    pub new_profile_screen: bool,

    /// Does a profile already exists?
    /// 
    /// We lookup for a `profile.data` file in the current directory of the executable
    pub profile_exists: bool,

    /// Currently Copy/Pasted private key (Used in Import Wallet button)
    pub private_key: String,

    /// Current input Name of a wallet (Used in New Wallet button)
    pub wallet_name: String,

    /// Confirm credentials for exporting a private key
    pub confirm_credentials: Credentials,

}

impl AppData {
    pub fn add_rpc(&mut self, rpc: Rpc) {
        self.rpc.push(rpc);
    }

    /// Save the rpc endpoints to `rpc.json`
    pub fn save_rpc(&self) -> Result<(), anyhow::Error> {
        let data = serde_json::to_string(&self.rpc.clone())?;
        std::fs::write("rpc.json", data)?;
        Ok(())
    }

    /// Load the rpc endpoints from file
    pub fn load_rpc(&mut self) -> Result<(), anyhow::Error> {
        let data = std::fs::read_to_string("rpc.json")?;
        self.rpc = serde_json::from_str(&data)?;
        Ok(())
    }

    /// Get native balance of a wallet address on the current chain
    pub fn native_balance(&mut self) -> String {
        // TODO get balance
        "0".to_string()
    }

    /// Get native coin on current chain
    pub fn native_coin(&self) -> String {
        match self.chain_id {
            ChainId::Ethereum(_) => "ETH".to_string(),
            ChainId::BinanceSmartChain(_) => "BNB".to_string(),
            ChainId::Base(_) => "ETH".to_string(),
            ChainId::Arbitrum(_) => "ETH".to_string(),
        }
    }

}


impl Default for AppData {
    fn default() -> Self {
        let profile_exists = Path::new("profile.data").exists();
        let new_profile_screen = !profile_exists;

        // Just to init AppData, we load the actual saved data later when we start ZeusApp
        let networks = vec![(ChainId::Ethereum(1), "Ethereum"), (ChainId::BinanceSmartChain(56), "Binance Smart Chain"), (ChainId::Base(8453), "Base"), (ChainId::Arbitrum(42161), "Arbitrum")];
        let mut rpc = vec![];

        for chain_id in networks.iter().map(|(chain_id, _)| chain_id.clone()){
            rpc.push(Rpc::new("".to_string(), chain_id));
        }

        

        Self {
            ws_client: None,
            chain_id: ChainId::default(),
            networks,
            rpc,
            profile: Profile::default(),
            tx_settings: TxSettings::default(),
            logged_in: false,
            new_profile_screen,
            profile_exists,
            private_key: "".to_string(),
            wallet_name: "".to_string(),
            confirm_credentials: Credentials::default(),
        }
    }
}