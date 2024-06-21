use alloy::{providers::{RootProvider, Provider}, pubsub::PubSubFrontend, primitives::U256, rpc::types::eth::Block};
use std::sync::Arc;
use serde::{Serialize, Deserialize};

pub mod forked_db;
pub mod defi;
pub mod app_state;
pub mod profile;

/// Websocket client
pub type WsClient = RootProvider<PubSubFrontend>;


/// Holds Block basic information
#[derive(Debug, Clone)]
pub struct BlockInfo {
    pub full_block: Option<Block>,
    pub number: u64,
    pub timestamp: u64,
    pub base_fee: U256
}

impl Default for BlockInfo {
    fn default() -> Self {
        Self {
            full_block: None,
            number: 0,
            timestamp: 0,
            base_fee: U256::default()
        }
    }
}

impl BlockInfo {
    pub fn new(full_block: Option<Block>, number: u64, timestamp: u64, base_fee: U256) -> Self {
        Self {
            full_block,
            number,
            timestamp,
            base_fee
        }
    }

    /// Wei to Gwei conversion
    pub fn gwei(&self) -> U256 {
        self.base_fee * U256::from(10).pow(U256::from(9))
    }

    /// Convert to human readable format
    pub fn readable(&self) -> String {
        format!("{:.2} Gwei", self.gwei() / U256::from(10).pow(U256::from(18)))
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChainId {
    Ethereum(u64),
    BinanceSmartChain(u64),
    Base(u64),
    Arbitrum(u64),
}

impl Default for ChainId {
    fn default() -> Self {
        Self::Ethereum(1)
    }
}

impl ChainId {

    pub async fn new(client: Arc<RootProvider<PubSubFrontend>>) -> Result<Self, anyhow::Error> {
        let chain_id = client.get_chain_id().await?;
        match chain_id {
            1 => Ok(Self::Ethereum(1)),
            56 => Ok(Self::BinanceSmartChain(56)),
            8453 => Ok(Self::Base(8453)),
            42161 => Ok(Self::Arbitrum(42161)),
            _ => Err(anyhow::anyhow!("Unsupported chain id: {}", chain_id)),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Ethereum(_) => "Ethereum".to_string(),
            Self::BinanceSmartChain(_) => "Binance Smart Chain".to_string(),
            Self::Base(_) => "Base".to_string(),
            Self::Arbitrum(_) => "Arbitrum".to_string(),
        
    }
}

    pub fn id(&self) -> u64 {
        match self {
            Self::Ethereum(id) => *id,
            Self::BinanceSmartChain(id) => *id,
            Self::Base(id) => *id,
            Self::Arbitrum(id) => *id,
        
    }
}
}


/// Rpc Url and [ChainId]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rpc {
    pub url: String,
    pub chain_id: ChainId,
}

impl Rpc {

    pub fn new(url: String, chain_id: ChainId) -> Self {
        Self { url, chain_id }
    }

    pub fn chain_name(&self) -> String {
        self.chain_id.name()
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, anyhow::Error> {
        serde_json::to_string(self).map_err(|e| anyhow::anyhow!(e))
    }
    

}

impl Default for Rpc {
    fn default() -> Self {
        Self {
            url: "wss://localhost:8545".to_string(),
            chain_id: ChainId::default(),
        }
    }
}