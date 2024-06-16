use alloy::primitives::{ Address, U256, Bytes };
use alloy::{ providers::RootProvider, pubsub::PubSubFrontend };
use zeus_defi::erc20::ERC20Token;

use std::sync::Arc;

use zeus_types::{ChainId, profile::Profile, WsClient, Rpc};

/// Request received from the frontend
pub enum Request {
    /// Simulate a swap
    SimSwap {
        /// Parameters needed to simulate a swap
        params: SwapParams,
    },

    /// Get the balance of an address
    Balance { address: Address},

    /// Encrypt and save the profile
    SaveProfile { profile: Profile },

    GetClient { chain_id: ChainId, rpcs: Vec<Rpc> },
}

/// The response from the backend
pub enum Response {

    SimSwap {result: SwapResult},

    Balance(U256),

    SaveProfile(Result<(), anyhow::Error>),

    GetClient(Result<WsClient, anyhow::Error>),
}

/// Parameters needed to simulate a swap
pub struct SwapParams {
    /// Chain id, if None will fetch from the client
    pub chain_id: Option<ChainId>,

    /// Client to make rpc calls
    pub client: Arc<RootProvider<PubSubFrontend>>,

    /// Address of the token we want to swap
    pub token_in: Address,

    /// Address of the token we want to get
    pub token_out: Address,

    /// Amount of tokens we want to swap
    pub amount_in: U256,

    /// Address of the caller
    pub caller: Address,

    /// Slippage
    pub slippage: String,
}



#[derive(Debug, Clone)]
/// The result of a simulated swap
pub struct SwapResult {

    pub token_in: ERC20Token,

    pub token_out: ERC20Token,

    /// Amount of token_in we sent
    pub amount_in: U256,

    /// Amount of token_out we got
    pub amount_out: U256,

    /// Minimum amount of tokens we may receive after slippage
    pub minimum_received: U256,

    /// Was the simulation succesful?
    pub success: bool,

    /// EVM Error message if the simulation failed
    pub evm_err: Vec<String>,

    /// A Generic Error
    pub error: String,

    /// Gas used
    pub gas_used: u64,

    /// Call Data to be used for the transaction
    pub data: Bytes,
}

impl SwapResult {
    pub fn from_err(err: String) -> Self {
        Self {
            token_in: ERC20Token::default(),
            token_out: ERC20Token::default(),
            amount_in: U256::from(0),
            amount_out: U256::from(0),
            minimum_received: U256::from(0),
            success: false,
            evm_err: Vec::new(),
            error: err,
            gas_used: 0,
            data: Bytes::new(),
        }
    }
}

impl Default for SwapResult {
    fn default() -> Self {
        Self {
            token_in: ERC20Token::default(),
            token_out: ERC20Token::default(),
            amount_in: U256::from(0),
            amount_out: U256::from(0),
            minimum_received: U256::from(0),
            success: false,
            evm_err: vec![],
            error: String::new(),
            gas_used: 0,
            data: Bytes::new(),
        }
    }
}