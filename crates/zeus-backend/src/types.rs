use std::collections::HashMap;
use std::sync::Arc;

use zeus_chain::{
    alloy::{
        primitives::{Address, U256},
        providers::RootProvider,
        pubsub::PubSubFrontend,
        rpc::types::eth::Block,
    },
    defi_types::currency::erc20::ERC20Token,
    ChainId, Rpc, WsClient,
};
use zeus_core::Profile;

pub struct EthBalanceParams {
    pub owner: Address,
    pub chain_id: u64,
    pub block: u64,
    pub client: Arc<WsClient>
}

pub struct ERC20BalanceParams {
    pub token: ERC20Token,
    pub owner: Address,
    pub chain_id: u64,
    pub block: u64,
    pub client: Arc<WsClient>
}
pub struct ERC20TokenParams {
    pub currency_id: String,
    pub owner: Address,
    pub token: Address,
    pub chain_id: u64,
    pub client: Arc<WsClient>
}

pub struct ERC20BalanceRes {
    pub owner: Address,
    pub token: Address,
    pub balance: U256,
    pub chain_id: u64
}

pub struct ERC20TokenRes {
    pub currency_id: String,
    pub owner: Address,
    pub token: ERC20Token,
    pub balance: U256,
    pub chain_id: u64
}


/// Request received from the frontend
pub enum Request {
    /// Thing to do on the startup of the application
    ///
    /// For now we just connect on the default chain and initialize the oracles
    OnStartup(ChainId, Vec<Rpc>),

    /// Initialize the Oracles
    InitOracles(Arc<WsClient>, ChainId),

    /// Get the eth balance of an address on a chain at a specific block
    EthBalance(EthBalanceParams),

    /// Get the ERC20 Balance
    ERC20Balance(ERC20BalanceParams),

    /// Encrypt and save the profile
    SaveProfile(Profile),

    Client(ChainId, Vec<Rpc>),

    ERC20Token(ERC20TokenParams)

}

impl Request {

    pub fn client(chain_id: ChainId, rpcs: Vec<Rpc>) -> Self {
        Request::Client(chain_id, rpcs)
    }

    pub fn on_startup(chain_id: ChainId, rpcs: Vec<Rpc>) -> Self {
        Request::OnStartup(chain_id, rpcs)
    }

    pub fn init_oracles(client: Arc<WsClient>, chain_id: ChainId) -> Self {
        Request::InitOracles(client, chain_id)
    }

    pub fn erc20_token(currency_id: String, owner: Address, token: Address, chain_id: u64, client: Arc<WsClient>) -> Self {
        Request::ERC20Token(ERC20TokenParams {
            currency_id,
            owner,
            token,
            chain_id,
            client
        })
    }

    pub fn eth_balance(owner: Address, chain_id: u64, block: u64, client: Arc<WsClient>) -> Self {
        Request::EthBalance(EthBalanceParams {
            owner,
            chain_id,
            block,
            client
        })
    }

    pub fn erc20_balance(token: ERC20Token, owner: Address, chain_id: u64, block: u64, client: Arc<WsClient>) -> Self {
        Request::ERC20Balance(ERC20BalanceParams {
            token,
            owner,
            chain_id,
            block,
            client
        })
    }
}

/// The response from the backend
pub enum Response {
    EthBalance(U256),

    Client(Option<Arc<WsClient>>, ChainId),

    ERC20Token(ERC20TokenRes),

    ERC20Balance(ERC20BalanceRes)
}

impl Response {

    pub fn eth_balance(balance: U256) -> Self {
        Response::EthBalance(balance)
    }

    pub fn client(client: Option<Arc<WsClient>>, chain_id: ChainId) -> Self {
        Response::Client(client, chain_id)
    }

    pub fn erc20_token(currency_id: String, owner: Address, token: ERC20Token, balance: U256, chain_id: u64) -> Self {
        Response::ERC20Token(ERC20TokenRes {
            currency_id,
            owner,
            token,
            balance,
            chain_id
        })
    }

    pub fn erc20_balance(owner: Address, token: Address, balance: U256, chain_id: u64) -> Self {
        Response::ERC20Balance(ERC20BalanceRes {
            owner,
            token,
            balance,
            chain_id
        })
    }
}