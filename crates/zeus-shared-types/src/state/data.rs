use std::{ path::Path, str::FromStr };
use std::sync::Arc;
use std::collections::HashMap;

use zeus_core::{anyhow, Profile, Credentials};
use zeus_chain::{alloy::primitives::{U256, Address}, ChainId, Rpc, BlockInfo, WsClient, serde_json};
use tracing::{info, trace};

/// Supported networks
pub const NETWORKS: [ChainId; 4] = [
    ChainId::Ethereum(1),
    ChainId::BinanceSmartChain(56),
    ChainId::Base(8453),
    ChainId::Arbitrum(42161),
];


/// Transaction settings
#[derive(Clone)]
pub struct TxSettings {
    pub priority_fee: String,
    pub slippage: String,
    pub mev_protect: bool,
}

impl TxSettings {
    /// Parse a wei from string to gwei
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

    pub latest_block: BlockInfo,

    pub next_block: BlockInfo,

    /// A map of all connected websocket clients
    pub ws_client: HashMap<u64, Arc<WsClient>>,

    /// The current selected chain id
    pub chain_id: ChainId,

    /// All supported ChainIds
    pub chain_ids: Vec<ChainId>,

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
    /// Get current client
    pub fn client(&self) -> Option<Arc<WsClient>> {
        self.ws_client.get(&self.chain_id.id()).cloned()
    }

    pub fn supported_networks(&self) -> Vec<u64> {
        self.chain_ids
            .iter()
            .cloned()
            .map(|chain_id| chain_id.id())
            .collect()
    }

    /// Are we connected to the provided chain id?
    ///
    /// We check if a ws_client exists for the provided chain id
    // ! Not 100% reliable as we may lose connection to the client
    pub fn connected(&self, chain_id: u64) -> bool {
        self.ws_client.contains_key(&chain_id)
    }

    /// Return the latest block
    pub fn latest_block(&self) -> BlockInfo {
        self.latest_block.clone()
    }

    /// Return the next block
    pub fn next_block(&self) -> BlockInfo {
        self.next_block.clone()
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

    /// Get eth balance of the a wallet for a specific chain
    pub fn eth_balance(&self, chain_id: u64) -> U256 {
        let current_wallet = if let Some(wallet) = &self.profile.current_wallet {
            wallet
        } else {
            trace!("No current wallet found");
            return U256::ZERO;
        };

        current_wallet.get_balance(chain_id)
    }

    /// DEBUG
    pub fn debug_wallet(&self) {
        if let Some(wallet) = &self.profile.current_wallet {
            trace!("Wallet Name: {:?}", wallet.name);
            trace!("Wallet Balance: {:?}", wallet.balance);
        } else {
            trace!("No current wallet found");
        }
    }


    /// Get the current wallet address
    pub fn wallet_address(&self) -> Address {
        if let Some(wallet) = &self.profile.current_wallet {
            wallet.key.address()
        } else {
            Address::ZERO
        }
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
            rpc.push(Rpc::new("".to_string(), chain_id.id()));
        }

        Self {
            latest_block: BlockInfo::default(),
            next_block: BlockInfo::default(),
            ws_client: HashMap::new(),
            chain_id: ChainId::default(),
            chain_ids: NETWORKS.to_vec(),
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