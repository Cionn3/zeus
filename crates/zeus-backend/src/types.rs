use alloy::primitives::{ Address, U256, Bytes };
use alloy::{ providers::RootProvider, pubsub::PubSubFrontend, transports::{TransportErrorKind, RpcError} };
use zeus_defi::erc20::ERC20Token;
use zeus_types::BlockInfo;

use std::collections::HashMap;
use std::sync::Arc;

use zeus_types::{ChainId, profile::Profile, WsClient, Rpc};




/// The result of a client request
#[derive(Debug)]
pub struct ClientResult {
    pub client: Arc<WsClient>,
    pub chain_id: ChainId,
}

/// Request received from the frontend
pub enum Request {

    /// Thing to do on the startup of the application
    /// 
    /// For now we just connect on the default chain and initialize the oracles
    OnStartup { chain_id: ChainId, rpcs: Vec<Rpc> },

    /// Initialize the Oracles
    InitOracles { client: Arc<WsClient>, chain_id: ChainId},

    /// Simulate a swap
    SimSwap {
        /// Parameters needed to simulate a swap
        params: SwapParams,
    },

    /// Get the eth balance of an address
    EthBalance { address: Address, client: Arc<WsClient>},

    /// Get ERC20 Balance
    GetERC20Balance { address: Address, token: Address, client: Arc<WsClient> },

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

    InitOracles(Result<(), anyhow::Error>),

    SimSwap {result: SwapResult},

    EthBalance(Result<U256, RpcError<TransportErrorKind>>),

    SaveProfile(Result<(), anyhow::Error>),

    GetClient(Result<ClientResult, anyhow::Error>),

    GetERC20Token(Result<(ERC20Token, String), anyhow::Error>),

    GetBlockInfo((BlockInfo, BlockInfo)),
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