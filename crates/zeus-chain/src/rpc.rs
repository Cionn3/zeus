use serde::{Deserialize, Serialize};



/// Rpc Url
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rpc {
    pub url: String,
    pub chain_id: u64,
}

impl Rpc {

    pub fn new(url: String, chain_id: u64) -> Self {
        Self { url, chain_id }
    }

    pub fn chain_name(&self) -> String {
        match self.chain_id {
            1 => "Ethereum".to_string(),
            56 => "Binance Smart Chain".to_string(),
            8453 => "Base".to_string(),
            42161 => "Arbitrum".to_string(),
            _ => "Unknown Chain ID".to_string(),
        }
    }

    pub fn is_url_empty(&self) -> bool {
        self.url.is_empty()
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, anyhow::Error> {
        serde_json::to_string(self).map_err(|e| anyhow::anyhow!(e))
    }
    

}

impl Default for Rpc {
    fn default() -> Self {
        Self {
            url: "wss://localhost:8546".to_string(),
            chain_id: 1,
        }
    }
}