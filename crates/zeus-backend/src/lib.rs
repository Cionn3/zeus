use alloy::{
    primitives::{U256, Address},
    providers::{ Provider, ProviderBuilder },
    transports::ws::WsConnect,
    rpc::types::eth::{ BlockId, BlockNumberOrTag },
};

use std::sync::Arc;
use tokio::{runtime::Runtime, sync::RwLock};
use crossbeam::channel::{ Sender, Receiver };
use revm::{ primitives::{ Bytes, TransactTo }, Evm, db::{ CacheDB, EmptyDB } };
use anyhow::{anyhow, Context};

use zeus_defi::{
    dex::uniswap::pool::{ get_v2_pool, get_v3_pools },
    erc20::ERC20Token,
    zeus_router::{ encode_swap, decode_swap, SwapRouter::Params, SWAP_ROUTER_ADDR },
};
use zeus_types::{ forked_db::fork_factory::ForkFactory, Rpc, WsClient };
use zeus_types::{ ChainId, profile::Profile, forked_db::{ fork_db::ForkDB, revert_msg } };

use zeus_utils::{get_client, new_evm, oracles::{OracleManager, OracleAction}};

use crate::{types::{ Request, Response, ClientResult, SwapParams, SwapResult }, db::ZeusDB};

pub mod types;
pub mod db;

/// A simple backend to handle async/expensive tasks without blocking the gui
///
/// All the API calls that the UI can make to the backend are defined here
///
/// Still in WIP
pub struct Backend {

    /// Send Data back to the frontend
    pub back_sender: Sender<Response>,

    /// Receive Data from the frontend
    pub front_receiver: Receiver<Request>,

    pub db: ZeusDB,

    /// The oracle manager
    pub oracle_manager: Option<Arc<RwLock<OracleManager>>>,

    pub client: Option<Arc<WsClient>>,
}

impl Backend {
    pub fn new(back_sender: Sender<Response>, front_receiver: Receiver<Request>) -> Self {
        Self {
            back_sender,
            front_receiver,
            db: ZeusDB::new().unwrap(),
            oracle_manager: None,
            client: None,
        }
    }

    /// Start the backend
    pub fn init(&mut self) {
        let rt = Runtime::new().unwrap();
        println!("Backend Started");

        rt.block_on(async {
            loop {
                match self.front_receiver.recv() {
                    Ok(request) => {
                        match request {

                            Request::OnStartup { chain_id, rpcs } => {
                                println!("On Startup");
                                self.get_client(chain_id.clone(), rpcs.clone()).await;
                            }

                            Request::InitOracles { client, chain_id } => {
                                self.init_oracle_manager(client, chain_id).await;
                            }

                            Request::SimSwap { params } => {
                                // TODO
                            }

                            Request::Balance { address } => {
                                // TODO
                            }

                            Request::SaveProfile { profile } => {
                                self.save_profile(profile);
                            }

                            Request::GetClient { chain_id, rpcs } => {
                                self.get_client(chain_id, rpcs).await;
                            }

                            Request::GetERC20Token { id, address, client, chain_id } => {
                                self.get_erc20_token(id, address, client, chain_id).await;
                            }
                        }
                    }
                    Err(_e) => {}
                }
            }
        })
    }

    async fn init_oracle_manager(&mut self, client: Arc<WsClient>, id: ChainId) {
        println!("Initializing Oracle Manager for Chain: {}", id.name());
        let ok_err: Result<(), anyhow::Error>;

        let oracle_manager = OracleManager::new(client, id.clone()).await;
        match oracle_manager {
            Ok(oracle_manager) => {
                self.handle_oracle().await;
                self.oracle_manager = Some(Arc::new(RwLock::new(oracle_manager)));
                self.start_oracles().await;
                ok_err = Ok(());
            }
            Err(e) => {
                println!("Error Initializing Oracle Manager: {}", e);
                ok_err = Err(e);
            }
        }
        
        match self.back_sender.send(Response::InitOracles(ok_err)) {
            Ok(_) => {}
            Err(e) => println!("Error Sending Response: {}", e),
        }
    }

    /// If we already run an oracle stop it so we can start a new one
    async fn handle_oracle(&mut self) {
        if let Some(oracle_manager) = &self.oracle_manager {
            let oracle_manager = oracle_manager.write().await;
            oracle_manager.action_sender.send(OracleAction::STOP).unwrap();
        }
    }

        async fn start_oracles(&mut self) {
            if let Some(oracle_manager) = &self.oracle_manager {
                let oracle_manager = oracle_manager.write().await;
                oracle_manager.start_oracles().await;
                println!("Oracles Started");
            }
        }
       


    /// Get the [ERC20Token] from the given address
    /// 
    /// If the token is not found in the database, we fetch it from the rpc
    /// 
    /// ### Arguments:
    /// 
    /// `id:` Which token to update in the UI ("token_in" or "token_out")
    /// 
    /// `address:` The address of the token
    /// 
    /// `client:` The websocket client
    /// 
    /// `chain_id:` The chain id
    async fn get_erc20_token(&self, id: String, address: Address, client: Arc<WsClient>, chain_id: u64) {
        let res = if let Ok(Some(token)) = self.db.get_erc20(address, chain_id) {
            Ok(token)
        } else {
            let token_result = ERC20Token::new(address, client, chain_id).await;
            match &token_result {
                Ok(token) => {
                    let token_to_insert = token.clone();
                    if let Err(e) = self.db.insert_erc20(token_to_insert, chain_id) {
                        println!("Error Inserting ERC20Token: {}", e);
                    }
                }
                Err(e) => println!("Error fetching ERC20Token from RPC: {}", e),
            }
            token_result
        };

        let res_converted = res
            .map(|token| (token, id))
            .context("Failed to get ERC20Token");

        match self.back_sender.send(Response::GetERC20Token(res_converted)) {
            Ok(_) => {}
            Err(e) => println!("Error Sending Response: {}", e),
        }
    }
    

    fn save_profile(&self, profile: Profile) {
        let res = profile.encrypt_and_save();

        match self.back_sender.send(Response::SaveProfile(res)) {
            Ok(_) => {}
            Err(e) => println!("Error Sending Response: {}", e),
        }
    }

    async fn get_client(&mut self, id: ChainId, rpcs: Vec<Rpc>)  {
        let url = rpcs.iter().find(|rpc| rpc.chain_id == id).unwrap().url.clone();

        let client_res = ProviderBuilder::new().on_ws(WsConnect::new(url)).await;
        let client_res = client_res.map(|client| ClientResult { client: client.into(), chain_id: id }).context("Failed to get client");

        match self.back_sender.send(Response::GetClient(client_res)) {
            Ok(_) => {}
            Err(e) => println!("Error Sending Response: {}", e),
        }
    }
}

/// Dummy implementation
async fn get_swap_result(params: SwapParams) -> Result<SwapResult, anyhow::Error> {
    let block_num = params.client.get_block_number().await?;
    let block_id = BlockId::Number(BlockNumberOrTag::Number(block_num));
    let cache_db = CacheDB::new(EmptyDB::default());

    let fork_factory = ForkFactory::new_sandbox_factory(
        params.client.clone(),
        cache_db,
        Some(block_id)
    );
    let fork_db = fork_factory.new_sandbox_fork();

    let block = params.client.get_block_by_number(BlockNumberOrTag::Number(block_num), true).await?;

    let block = if let Some(block) = block {
        block
    } else {
        return Err(anyhow!("Block not found"));
    };

    let chain_id = ChainId::new(params.client.clone()).await?;

    let mut evm = new_evm(fork_db, block, chain_id.clone());
    let result = swap(chain_id, params, &mut evm).await?;
    Ok(result)
}

/// Simulate a swap on Uniswap V2/V3
///
/// The pool with the highest output is selected
async fn swap(
    chain_id: ChainId,
    params: SwapParams,
    evm: &mut Evm<'static, (), ForkDB>
) -> Result<SwapResult, anyhow::Error> {
    let client = params.client;
    let slippage: u32 = params.slippage.parse().unwrap_or(1);

    let token_in = ERC20Token::new(params.token_in, client.clone(), chain_id.id()).await?;

    let token_out = ERC20Token::new(params.token_out, client.clone(), chain_id.id()).await?;

    let v2_pool = get_v2_pool(
        token_in.clone(),
        token_out.clone(),
        chain_id.clone(),
        client.clone()
    ).await?;

    let mut pools = get_v3_pools(
        token_in.clone(),
        token_out.clone(),
        chain_id,
        client.clone()
    ).await?;

    if let Some(v2_pool) = v2_pool {
        pools.push(v2_pool);
    }

    if pools.is_empty() {
        return Err(anyhow!("No pools found"));
    }

    let mut highest_amount_out = U256::ZERO;
    let mut gas_used: u64 = 0;
    let mut success = false;
    let mut evm_err = Vec::new();
    let mut call_data = Bytes::default();

    // approve the contract to spend token_in
    evm.tx_mut().caller = params.caller;
    evm.tx_mut().transact_to = TransactTo::Call(token_in.address);
    evm.tx_mut().value = U256::ZERO;
    evm.tx_mut().data = token_in.encode_approve(*SWAP_ROUTER_ADDR, params.amount_in).into();

    let res = evm.transact_commit()?;

    if !res.is_success() {
        let err = revert_msg(&res.output().unwrap_or_default());
        return Err(anyhow!(err));
    }

    evm.tx_mut().transact_to = TransactTo::Call(*SWAP_ROUTER_ADDR);

    for pool in pools {
        let mut router_params = Params {
            input_token: token_in.address,
            output_token: token_out.address,
            amount_in: params.amount_in,
            pool: pool.address,
            pool_variant: pool.variant(),
            minimum_received: U256::ZERO,
        };

        let data = encode_swap(router_params.clone());

        evm.tx_mut().data = data.clone();
        let res = evm.transact()?.result;
        let output = res.clone().into_output().unwrap_or_default();
        let amount_out = decode_swap(output);

        if res.is_success() && amount_out > highest_amount_out {
            highest_amount_out = amount_out;
            gas_used = res.gas_used();
            success = res.is_success();

            let minimum_received =
                amount_out - (amount_out * U256::from(slippage)) / U256::from(100);

            // update the calldata
            router_params.minimum_received = minimum_received;
            call_data = encode_swap(router_params);
        }

        if !res.is_success() {
            let err = revert_msg(&res.output().unwrap_or_default());
            evm_err.push(err);
        }
    }

    // no swaps were successful
    if highest_amount_out == U256::ZERO {
        return Ok(SwapResult {
            token_in,
            token_out,
            amount_in: params.amount_in,
            amount_out: U256::ZERO,
            minimum_received: U256::ZERO,
            success: false,
            evm_err,
            error: "".to_string(),
            gas_used,
            data: call_data,
        });
    }

    // TODO avoid calculating this twice
    let minimum_received =
        highest_amount_out - (highest_amount_out * U256::from(slippage)) / U256::from(100);

    return Ok(SwapResult {
        token_in,
        token_out,
        amount_in: params.amount_in,
        amount_out: highest_amount_out,
        minimum_received,
        success,
        evm_err,
        error: "".to_string(),
        gas_used,
        data: call_data,
    });
}
