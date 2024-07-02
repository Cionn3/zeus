use alloy::{
    providers::{RootProvider, Provider},
    pubsub::PubSubFrontend,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};



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