use super::chain_id::ChainId;
use serde::{Deserialize, Serialize};



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