use std::{path::Path, str::FromStr};
use std::sync::Arc;
use std::collections::HashMap;
use crate::{WsClient, BlockInfo};
use alloy::primitives::{U256, Address};


use crate::{profile::{Credentials, Profile}, ChainId, Rpc};

/// Supported networks
pub const NETWORKS: [ChainId; 4] = [
    ChainId::Ethereum(1),
    ChainId::BinanceSmartChain(56),
    ChainId::Base(8453),
    ChainId::Arbitrum(42161),
];

/// Hold ERC20 token balances for a given block
pub struct ERC20Balances {
    pub balances: HashMap<u64, HashMap<Address, U256>>,
}

impl Default for ERC20Balances {
    fn default() -> Self {
        Self {
            balances: HashMap::new(),
        }
    }
}

impl ERC20Balances {
    pub fn update_balance(&mut self, block_number: u64, token: Address, balance: U256) {
        self.balances
            .entry(block_number)
            .or_insert_with(HashMap::new)
            .insert(token, balance);
        
        // Remove all blocks older than the current block number
        self.balances.retain(|&k, _| k == block_number);
    }

    pub fn get_balance(&self, block_number: u64, token: &Address) -> Option<&U256> {
        self.balances
            .get(&block_number)
            .and_then(|token_balances| token_balances.get(token))
    }
}

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

pub struct AppData {

    /// Cache ERC20 balances for a given block
    pub erc20_balances: ERC20Balances,

    pub block_info: (BlockInfo, BlockInfo),

    /// A map of all connected websocket clients
    pub ws_client: HashMap<u64, Arc<WsClient>>,

    /// The current selected chain id
    pub chain_id: ChainId,

    /// All supported networks
    pub networks: Vec<ChainId>,

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

    pub fn supported_networks(&self) -> Vec<u64> {
        self.networks.iter().cloned().map(|chain_id| chain_id.id()).collect()
    }

    /// Are we connected to the provided chain id?
    /// 
    /// We check if a ws_client exists for the provided chain id
    // ! Not 100% reliable as we may lose connection to the client
    pub fn connected(&self, chain_id: u64) -> bool {
        self.ws_client.contains_key(&chain_id)
    }

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
        let mut rpc = vec![];

        for chain_id in NETWORKS {
            rpc.push(Rpc::new("".to_string(), chain_id));
        }

        

        Self {
            block_info: (BlockInfo::default(), BlockInfo::default()),
            erc20_balances: ERC20Balances::default(),
            ws_client: HashMap::new(),
            chain_id: ChainId::default(),
            networks: NETWORKS.to_vec(),
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