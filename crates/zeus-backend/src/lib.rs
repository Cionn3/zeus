use alloy::{
    primitives::{ U256, Address },
    providers::{ Provider, ProviderBuilder },
    transports::ws::WsConnect,
    rpc::types::eth::{ BlockId, BlockNumberOrTag },
};

use std::sync::Arc;
use tokio::{ runtime::Runtime, sync::{ RwLock as tokioRwLock, Mutex } };
use crossbeam::channel::{ Sender, Receiver };
use revm::{ primitives::{ Bytes, TransactTo }, Evm, db::{ CacheDB, EmptyDB } };
use anyhow::{ anyhow, Context };
use tracing::{ info, error, trace };

use zeus_types::{
    app_state::state::*,
    defi::{
        dex::uniswap::
            pool::{ get_v2_pool, get_v3_pool, V3_FEES, PoolVariant, Pool },
        erc20::ERC20Token,
        zeus_router::{ decode_swap, encode_swap, SwapRouter::Params, swap_router_bytecode },
    },
    forked_db::{ fork_db::ForkDB, fork_factory::ForkFactory },
    profile::Profile,
    ChainId,
    Rpc,
    WsClient,
};

use zeus_utils::{
    new_evm,
    oracles::{ OracleManager, OracleAction },
    dummy_account::*,
    parse_ether,
};

use crate::{ types::{ Request, Response, SwapParams }, db::ZeusDB };

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
    pub oracle_manager: Option<Arc<tokioRwLock<OracleManager>>>,
}

impl Backend {
    pub fn new(back_sender: Sender<Response>, front_receiver: Receiver<Request>) -> Self {
        Self {
            back_sender,
            front_receiver,
            db: ZeusDB::new().unwrap(),
            oracle_manager: None,
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
                                match self.get_client(chain_id.clone(), rpcs.clone()).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        let mut state = SHARED_UI_STATE.write().unwrap();
                                        state.err_msg = ErrorMsg::new(true, e);
                                    }
                                }
                            }

                            Request::InitOracles { client, chain_id } => {
                                match self.init_oracle_manager(client, chain_id).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        let mut state = SHARED_UI_STATE.write().unwrap();
                                        state.err_msg = ErrorMsg::new(true, e);
                                    }
                                }
                            }

                            Request::GetBlockInfo {} => {
                                self.get_block_info().await;
                            }

                            Request::GetERC20Balance {
                                id,
                                token,
                                owner,
                                chain_id,
                                block,
                                client,
                            } => {
                                match
                                    self.get_erc20_balance(
                                        id,
                                        token,
                                        owner,
                                        chain_id,
                                        block,
                                        client
                                    ).await
                                {
                                    Ok(_) => {}
                                    Err(e) => {
                                        let mut state = SHARED_UI_STATE.write().unwrap();
                                        state.err_msg = ErrorMsg::new(true, e);
                                    }
                                }
                            }

                            Request::GetQuoteResult { params } => {
                                match self.get_swap_result(params).await {
                                    Ok(result) => {
                                        let mut swap_state = SWAP_UI_STATE.write().unwrap();
                                        swap_state.quote_result = result;
                                    }
                                    Err(e) => {
                                        let mut state = SHARED_UI_STATE.write().unwrap();
                                        state.err_msg = ErrorMsg::new(true, e);
                                    }
                                }
                            }

                            Request::EthBalance { address, client } => {
                                match self.get_eth_balance(address, client).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        let mut state = SHARED_UI_STATE.write().unwrap();
                                        state.err_msg = ErrorMsg::new(true, e);
                                    }
                                }
                            }

                            Request::SaveProfile { profile } => {
                                match self.save_profile(profile) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        let mut state = SHARED_UI_STATE.write().unwrap();
                                        state.err_msg = ErrorMsg::new(true, e);
                                    }
                                }
                            }

                            Request::GetClient { chain_id, rpcs, clients } => {
                                if !clients.contains_key(&chain_id.id()) {
                                    match self.get_client(chain_id, rpcs).await {
                                        Ok(_) => {}
                                        Err(e) => {
                                            let mut state = SHARED_UI_STATE.write().unwrap();
                                            state.err_msg = ErrorMsg::new(true, e);
                                        }
                                    }
                                } else {
                                    let client = clients.get(&chain_id.id()).unwrap().clone();
                                    match
                                        self.back_sender.send(Response::GetClient(client, chain_id))
                                    {
                                        Ok(_) => {}
                                        Err(e) => println!("Error Sending Response: {}", e),
                                    }
                                }
                            }

                            Request::GetERC20Token { id, address, client, chain_id } => {
                                match self.get_erc20_token(id, address, client, chain_id).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        let mut state = SHARED_UI_STATE.write().unwrap();
                                        state.err_msg = ErrorMsg::new(true, e);
                                    }
                                }
                            }
                        }
                    }
                    Err(_e) => {}
                }
            }
        })
    }

    async fn init_oracle_manager(
        &mut self,
        client: Arc<WsClient>,
        id: ChainId
    ) -> Result<(), anyhow::Error> {
        println!("Initializing Oracle Manager for Chain: {}", id.name());

        let oracle_manager = OracleManager::new(client, id.clone()).await?;
        self.handle_oracle().await;
        self.oracle_manager = Some(Arc::new(tokioRwLock::new(oracle_manager)));
        self.start_oracles().await;
        Ok(())
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

    async fn get_block_info(&self) {
        if let Some(oracle_manager) = &self.oracle_manager {
            let oracle = oracle_manager.read().await;
            let (latest_block, next_block) = oracle.get_block_info().await;
            match self.back_sender.send(Response::GetBlockInfo((latest_block, next_block))) {
                Ok(_) => {}
                Err(e) => println!("Error Sending Response: {}", e),
            }
        }
    }

    async fn get_eth_balance(
        &mut self,
        address: Address,
        client: Arc<WsClient>
    ) -> Result<(), anyhow::Error> {
        let balance = client.get_balance(address).await?;
        self.back_sender.send(Response::EthBalance(balance))?;
        Ok(())
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
    async fn get_erc20_token(
        &self,
        id: String,
        address: Address,
        client: Arc<WsClient>,
        chain_id: u64
    ) -> Result<(), anyhow::Error> {
        let token = if let Ok(token) = self.db.get_erc20(address, chain_id) {
            token
        } else {
            let token = ERC20Token::new(address, client, chain_id).await?;
            self.db.insert_erc20(token.clone(), chain_id)?;
            token
        };
        let mut swap_ui_state = SWAP_UI_STATE.write().unwrap();

        // replace with the new token
        swap_ui_state.replace_token(&id, SelectedToken::new(token.clone()));

        // close the token list window
        swap_ui_state.update_token_list_status(&id, false);

        // update the token list HashMap
        if let Some(tokens) = swap_ui_state.tokens.get_mut(&chain_id) {
            tokens.push(token);
        } else {
            swap_ui_state.tokens.insert(chain_id, vec![token]);
        }
        Ok(())
    }

    /// Get the balance of an erc20 token
    async fn get_erc20_balance(
        &self,
        id: String,
        token: ERC20Token,
        owner: Address,
        chain_id: u64,
        block: u64,
        client: Arc<WsClient>
    ) -> Result<(), anyhow::Error> {
        // check if the balance is in the database
        let balance = if
            let Ok(balance) = self.db.get_erc20_balance(token.address, chain_id, block)
        {
            balance
        } else {
            let balance = token.balance_of(owner, client.clone()).await?;
            if let Err(_) = self.db.insert_erc20_balance(token.address, balance, chain_id, block) {
            }
            if let Err(_) = self.db.remove_erc20_balance(block, chain_id) {
            }
            balance
        };
        // update the balance
        let mut swap_state = SWAP_UI_STATE.write().unwrap();
        swap_state.update_balance(&id, balance.to_string());
        Ok(())
    }

    fn save_profile(&self, profile: Profile) -> Result<(), anyhow::Error> {
        profile.encrypt_and_save()?;
        Ok(())
    }

    async fn get_client(&mut self, id: ChainId, rpcs: Vec<Rpc>) -> Result<(), anyhow::Error> {
        let url = rpcs
            .iter()
            .find(|rpc| rpc.chain_id == id)
            .context(format!("Failed to find RPC for {}", id.name()))?
            .url.clone();

        let client = ProviderBuilder::new().on_ws(WsConnect::new(url)).await?;
        let client = Arc::new(client);

        self.back_sender.send(Response::GetClient(client, id))?;
        Ok(())
    }

    /// Dummy implementation
    async fn get_swap_result(&self, params: SwapParams) -> Result<QuoteResult, anyhow::Error> {
        let block_id = BlockId::Number(
            BlockNumberOrTag::Number(params.block.header.number.unwrap())
        );
        let cache_db = CacheDB::new(EmptyDB::default());

        let mut fork_factory = ForkFactory::new_sandbox_factory(
            params.client.clone(),
            cache_db,
            Some(block_id)
        );

        let dummy_caller = DummyAccount::new(
            AccountType::EOA,
            parse_ether("10")?,
            parse_ether("10")?
        );
        let dummy_contract = DummyAccount::new(
            AccountType::Contract(swap_router_bytecode()),
            U256::ZERO,
            U256::ZERO
        );
        if
            let Err(e) = insert_dummy_account(
                &dummy_caller,
                params.chain_id.clone(),
                &mut fork_factory
            )
        {
            return Err(e);
        }
        if
            let Err(e) = insert_dummy_account(
                &dummy_contract,
                params.chain_id.clone(),
                &mut fork_factory
            )
        {
            return Err(e);
        }
        let fork_db = fork_factory.new_sandbox_fork();

        let result = self.swap(dummy_contract, dummy_caller, params, fork_db).await?;
        Ok(result)
    }

    /// Simulate a swap on Uniswap V2/V3
    ///
    /// The pool with the highest output is selected
    async fn swap(
        &self,
        contract: DummyAccount,
        caller: DummyAccount,
        params: SwapParams,
        fork_db: ForkDB
    ) -> Result<QuoteResult, anyhow::Error> {
        let slippage: f32 = params.slippage.parse().unwrap_or(1.0);
        let amount_in = params.token_in.token.parse(&params.amount_in)?;

        let pools = self.collect_pools(params.clone()).await?;

        let best_pool = Arc::new(Mutex::new(pools[0].clone()));
        let best_amount_out = Arc::new(Mutex::new(U256::ZERO));

        let mut evm = new_evm(fork_db, params.block.clone(), params.chain_id.clone());

        // approve the contract to spend token_in
        evm.tx_mut().caller = caller.address;
        evm.tx_mut().transact_to = TransactTo::Call(params.token_in.token.address);
        evm.tx_mut().value = U256::ZERO;
        evm.tx_mut().data = params.token_in.token.encode_approve(contract.address, amount_in).into();

        evm.transact_commit()?;

        let fork_db = evm.db().clone();

        let time = std::time::Instant::now();
        let mut handles = Vec::new();
        for pool in pools {
            let pool = pool.clone();
            let params = params.clone();
            let fork_db = fork_db.clone();
            let contract = contract.clone();
            let caller = caller.clone();
            let best_pool = best_pool.clone();
            let best_amount_out = best_amount_out.clone();

            handles.push(tokio::spawn(async move {

                let amount_out;
                {
                let mut evm = new_evm(fork_db, params.block.clone(), params.chain_id.clone());
                amount_out = sim_swap(pool.clone(), contract, caller, params, &mut evm).unwrap();
                }
                let mut best_pool = best_pool.lock().await;
                let mut best_amount_out = best_amount_out.lock().await;
                if amount_out > *best_amount_out {
                    *best_amount_out = amount_out;
                    *best_pool = pool;
                }
            }));
        }

        for handle in handles {
            handle.await?;
        }
    
        info!("Time to swap: {:?}ms", time.elapsed().as_millis());

        let best_pool = best_pool.lock().await;
        let best_amount_out = best_amount_out.lock().await;
        let pool_to_swap = best_pool.clone();
        let amount_out = best_amount_out.clone();

        

        let minimum_received = amount_out - (amount_out * U256::from(slippage)) / U256::from(100);

        Ok(QuoteResult {
            block_number: params.block.header.number.unwrap(),
            input_token: params.token_in.clone(),
            output_token: params.token_out.clone(),
            input_token_usd_worth: "TODO".to_string(),
            output_token_usd_worth: "TODO".to_string(),
            price_impact: "TODO".to_string(),
            slippage: "TODO".to_string(),
            real_amount: amount_out.to_string(),
            minimum_received: minimum_received.to_string(),
            token_tax: "TODO".to_string(),
            pool_fee: "TODO".to_string(),
            gas_cost: "TODO".to_string(),
            data: Bytes::default(),
        })
    }

    

    async fn collect_pools(&self, params: SwapParams) -> Result<Vec<Pool>, anyhow::Error> {
        let pools = Arc::new(Mutex::new(Vec::new()));

        let v2_pool = if
            let Ok(pool) = self.db.get_pool(
                params.token_in.token.clone(),
                params.token_out.token.clone(),
                params.chain_id.id().clone(),
                PoolVariant::UniswapV2,
                3000
            )
        {
            Some(pool)
        } else {
            if
                let Ok(pool) = get_v2_pool(
                    params.token_in.token.clone(),
                    params.token_out.token.clone(),
                    params.chain_id.clone(),
                    params.client.clone()
                ).await
            {
                if let Err(e) = self.db.insert_pool(pool.clone(), params.chain_id.id()) {
                    trace!("Failed to insert pool into db {}", e);
                }
                Some(pool)
            } else {
                None
            }
        };

        for fee in V3_FEES {
            let params = params.clone();
            let pools = pools.clone();
            let db = self.db.clone();
            tokio::spawn(async move {
                // check db first
                if
                    let Ok(pool) = db.get_pool(
                        params.token_in.token.clone(),
                        params.token_out.token.clone(),
                        params.chain_id.id().clone(),
                        PoolVariant::UniswapV3,
                        fee
                    )
                {
                    let mut pools = pools.lock().await;
                    pools.push(pool);
                } else {
                    // not in db fetch from rpc
                    if
                        let Ok(pool) = get_v3_pool(
                            params.token_in.token.clone(),
                            params.token_out.token.clone(),
                            fee,
                            params.chain_id.clone(),
                            params.client.clone()
                        ).await
                    {
                        if let Err(e) = db.insert_pool(pool.clone(), params.chain_id.id()) {
                            error!("Failed to insert pool into db {}", e);
                        }
                        let mut pools = pools.lock().await;
                        pools.push(pool);
                    }
                }
            });
        }

        let mut pools = pools.lock().await;

        if v2_pool.is_some() {
            pools.push(v2_pool.unwrap());
        }

        if pools.is_empty() {
            return Err(anyhow!("No pools found"));
        }

        let all_pools = pools.iter().cloned().collect::<Vec<Pool>>();
        Ok(all_pools)
    }
}

fn sim_swap(
    pool: Pool,
    contract: DummyAccount,
    caller: DummyAccount,
    params: SwapParams,
    evm: &mut Evm<'static, (), ForkDB>
) -> Result<U256, anyhow::Error> {
    let amount_in = params.token_in.token.parse(&params.amount_in)?;

    // approve the contract to spend token_in
    evm.tx_mut().caller = caller.address;
    evm.tx_mut().transact_to = TransactTo::Call(params.token_in.token.address);
    evm.tx_mut().value = U256::ZERO;
    evm.tx_mut().transact_to = TransactTo::Call(contract.address);

    let router_params = Params {
        input_token: params.token_in.token.address,
        output_token: params.token_out.token.address,
        amount_in,
        pool: pool.address,
        pool_variant: pool.variant(),
        minimum_received: U256::ZERO,
    };

    let data = encode_swap(router_params);
    evm.tx_mut().data = data;

    let amount_out;

    let res = evm.transact().unwrap().result;
    let output = res.clone().into_output().unwrap_or_default();

    amount_out = if res.is_success() {
        info!("Sim Success");
        decode_swap(output).unwrap()
    } else {
        U256::ZERO
    };

    
    Ok(amount_out)
}