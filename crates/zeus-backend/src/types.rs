use alloy::primitives::{ Address, U256};
use alloy::{ providers::RootProvider, pubsub::PubSubFrontend, rpc::types::eth::Block };
use zeus_types::defi::erc20::ERC20Token;

use std::collections::HashMap;
use std::sync::Arc;

use zeus_types::{ChainId, profile::Profile, WsClient, Rpc, app_state::state::SelectedToken};



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

    GetERC20Token { id: String, owner: Address, address: Address, client: Arc<WsClient>, chain_id: u64 },

}

/// The response from the backend
pub enum Response {

    EthBalance(U256),

    GetClient(Arc<WsClient>, ChainId),

}

/// Parameters needed to simulate a swap
#[derive(Clone)]
pub struct SwapParams {
    /// The target Chain id
    pub chain_id: ChainId,

    /// Latest block
    pub block: Block,

    /// Client to make rpc calls
    pub client: Arc<RootProvider<PubSubFrontend>>,

    pub token_in: SelectedToken,

    pub token_out: SelectedToken,

    /// Amount of tokens we want to swap
    pub amount_in: String,

    /// Address of the caller
    pub caller: Address,

    /// Slippage
    pub slippage: String,
}