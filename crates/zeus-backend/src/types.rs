use alloy::primitives::{ Address, U256, Bytes };
use alloy::{ providers::RootProvider, pubsub::PubSubFrontend, rpc::types::eth::Block };
use zeus_types::{BlockInfo, defi::erc20::ERC20Token, app_state::state::QuoteResult};

use std::collections::HashMap;
use std::sync::Arc;

use zeus_types::{ChainId, profile::Profile, WsClient, Rpc};



/// Request received from the frontend
pub enum Request {

    /// Thing to do on the startup of the application
    /// 
    /// For now we just connect on the default chain and initialize the oracles
    OnStartup { chain_id: ChainId, rpcs: Vec<Rpc> },

    /// Initialize the Oracles
    InitOracles { client: Arc<WsClient>, chain_id: ChainId},

    /// Simulate a swap
    GetQuoteResult {
        /// Parameters needed to simulate a swap
        params: SwapParams,
    },

    /// Get the eth balance of an address
    EthBalance { address: Address, client: Arc<WsClient>},

    /// Get ERC20 Balance
    GetERC20Balance { id: String, token: ERC20Token, owner: Address, chain_id: u64, block: u64, client: Arc<WsClient> },

    /// Encrypt and save the profile
    SaveProfile { profile: Profile },

    GetClient { chain_id: ChainId, rpcs: Vec<Rpc>, clients: HashMap<u64, Arc<WsClient>> },

    GetERC20Token { id: String, address: Address, client: Arc<WsClient>, chain_id: u64 },

    /// Get the `latest_block` & `next_block` Info from the [crate::OracleManager]
    /// 
    /// No need to specify chain_id, since we update it every time we change a chain
    GetBlockInfo,

}

/// The response from the backend
pub enum Response {

    GetQuoteResult(QuoteResult),

    EthBalance(U256),

    GetClient(Arc<WsClient>, ChainId),

   // GetERC20Token(ERC20Token, String),

    GetBlockInfo((BlockInfo, BlockInfo)),
}

/// Parameters needed to simulate a swap
#[derive(Debug, Clone)]
pub struct SwapParams {
    /// The target Chain id
    pub chain_id: ChainId,

    /// Latest block
    pub block: Block,

    /// Client to make rpc calls
    pub client: Arc<RootProvider<PubSubFrontend>>,

    pub token_in: ERC20Token,

    pub token_out: ERC20Token,

    /// Amount of tokens we want to swap
    pub amount_in: String,

    /// Address of the caller
    pub caller: Address,

    /// Slippage
    pub slippage: String,
}