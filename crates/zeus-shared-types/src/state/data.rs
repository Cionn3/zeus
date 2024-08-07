use std::{ path::Path, str::FromStr };
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use zeus_core::{anyhow, Profile};
use zeus_chain::{alloy::primitives::{U256, Address}, ChainId, Rpc, BlockInfo, WsClient, serde_json};
use crate::cache::{SHARED_CACHE, SharedCache};
use tracing::trace;

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

    /// The current client
    pub client: Option<Arc<WsClient>>,

    /// Are we connected to the client?
    pub connected: bool,

    /// The current selected chain id
    pub chain_id: ChainId,

    /// All supported ChainIds
    pub chain_ids: Vec<ChainId>,

    /// The current saved RPC endpoints
    pub rpc: Vec<Rpc>,

    /// The current profile
    pub profile: Profile,

    pub shared_cache: Arc<RwLock<SharedCache>>,

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
}

impl AppData {
    /// Get current client
    pub fn client(&self) -> &Option<Arc<WsClient>> {
        &self.client
    }

    pub fn supported_networks(&self) -> Vec<u64> {
        self.chain_ids
            .iter()
            .cloned()
            .map(|chain_id| chain_id.id())
            .collect()
    }

    pub fn connected(&self) -> bool {
        self.client.is_some()
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

    /// Get eth balance of a wallet for a specific chain
    pub fn eth_balance(&self, chain_id: u64, owner: Address) -> (u64, U256) {
        self.shared_cache.read().unwrap().get_eth_balance(chain_id, owner)
    }

    /// Update eth balance of a wallet for a specific chain
    pub fn update_balance(&mut self, chain_id: u64, owner: Address, balance: U256) {
        let block = self.latest_block.number;
        self.shared_cache.write().unwrap().update_eth_balance(chain_id, owner, block, balance);
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
            client: None,
            connected: false,
            chain_id: ChainId::default(),
            chain_ids: NETWORKS.to_vec(),
            rpc,
            profile: Profile::default(),
            shared_cache: SHARED_CACHE.clone(),
            tx_settings: TxSettings::default(),
            logged_in: false,
            new_profile_screen,
            profile_exists,
        }
    }
}