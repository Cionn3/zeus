use eframe::{egui, CreationContext};
use egui::{Context, Style};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use crossbeam::channel::{unbounded, Receiver, Sender};

use crate::{
    fonts::get_fonts,
    gui::{
        misc::{show_err_msg, show_login, tx_settings_window},
        GUI,
    },
    theme::ZeusTheme,
};

use zeus_backend::{
    db::ZeusDB,
    types::{Request, Response},
    Backend,
};
use zeus_chain::{
    alloy::primitives::{Address, U256},
    defi_types::currency::Currency,
    BLOCK_ORACLE,
};
use zeus_shared_types::{cache::SHARED_CACHE, AppData, ErrorMsg, SHARED_UI_STATE, SWAP_UI_STATE};

use tracing_subscriber::{
    fmt, layer::SubscriberExt, prelude::*, util::SubscriberInitExt, EnvFilter,
};

use tracing::{error, info, trace};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::Registry;

/// The timeout for requesting eth and erc20 balances
/// This do not apply for Ethereum since it has a block time of 12 secs and cannot cause a lot of rpc calls
const TIME_OUT: u64 = 3;

/// The width of the window
pub const WIDTH: f32 = 1280.0;

/// The height of the window
pub const HEIGHT: f32 = 720.0;

/// The main application struct
pub struct ZeusApp {
    /// The GUI components of the application
    pub gui: GUI,

    /// Send Data to backend
    pub front_sender: Option<Sender<Request>>,

    /// Receive Data from backend
    pub back_receiver: Option<Receiver<Response>>,

    /// The app data of the application
    pub data: AppData,

    pub last_eth_request: Instant,

    pub last_erc20_request: Instant,

    pub last_quote_request: Instant,

    pub on_startup: bool,

    pub top_panel_h: f32,

    pub left_panel_w: f32,
}

fn setup_logging() -> (WorkerGuard, WorkerGuard) {
    // Setup for file appenders
    let trace_appender = tracing_appender::rolling::daily("./logs", "trace.log");
    let output_appender = tracing_appender::rolling::daily("./logs", "output.log");

    // Creating non-blocking writers
    let (trace_writer, trace_guard) = tracing_appender::non_blocking(trace_appender);
    let (output_writer, output_guard) = tracing_appender::non_blocking(output_appender);

    // Defining filters
    let console_filter =
        EnvFilter::new("zeus,zeus_core,zeus_chain,zeus_backend,zeus_shared_types=info,error,warn");
    let trace_filter =
        EnvFilter::new("zeus,zeus_core,zeus_chain,zeus_backend,zeus_shared_types=trace");
    let output_filter =
        EnvFilter::new("zeus,zeus_core,zeus_chain,zeus_backend,zeus_shared_types=info,error,warn");

    // Setting up layers
    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_filter(console_filter);
    let trace_layer = fmt::layer()
        .with_writer(trace_writer)
        .with_filter(trace_filter);
    let output_layer = fmt::layer()
        .with_writer(output_writer)
        .with_filter(output_filter);

    // Applying configuration
    Registry::default()
        .with(trace_layer)
        .with(console_layer)
        .with(output_layer)
        .init();

    (trace_guard, output_guard)
}

impl ZeusApp {
    pub fn new(cc: &CreationContext) -> Self {
        let _guard = setup_logging();

        let gui = GUI::default();

        let mut app = Self {
            gui,
            front_sender: None,
            back_receiver: None,
            data: AppData::default(),
            last_eth_request: Instant::now(),
            last_erc20_request: Instant::now(),
            last_quote_request: Instant::now(),
            on_startup: true,
            top_panel_h: 0.0,
            left_panel_w: 0.0,
        };

        let theme = app.config_style(&cc.egui_ctx);
        app.gui.theme = Arc::new(theme);

        match app.data.load_rpc() {
            Ok(_) => {}
            Err(e) => {
                error!("Error Loading rpc.json: {}", e);
            }
        }

        let currencies: HashMap<u64, Vec<Currency>>;
        let erc20_balances: HashMap<(u64, Address, Address), U256>;
        let eth_balances: HashMap<(u64, Address), (u64, U256)>;

        {
            let zeus_db = match ZeusDB::new() {
                Ok(db) => db,
                Err(e) => {
                    // TODO: handle this differently
                    error!("Error Creating Database: {}", e);
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.err_msg = ErrorMsg::new(true, e);
                    return app;
                }
            };

            match zeus_db.insert_default() {
                Ok(_) => {}
                Err(e) => {
                    error!("Error Inserting Default Tokens: {}", e);
                }
            }

            let networks = app.data.supported_networks();

            currencies = match zeus_db.load_currencies(networks.clone()) {
                Ok(currencies) => currencies,
                Err(e) => {
                    error!("Error Loading Currencies: {}", e);
                    HashMap::new()
                }
            };

            erc20_balances = match zeus_db.load_all_erc20_balances(networks.clone()) {
                Ok(balances) => balances,
                Err(e) => {
                    error!("Error Loading ERC20 Balances: {}", e);
                    HashMap::new()
                }
            };

            eth_balances = match zeus_db.load_all_eth_balances(networks) {
                Ok(balances) => balances,
                Err(e) => {
                    error!("Error Loading ETH Balances: {}", e);
                    HashMap::new()
                }
            };
            trace!("ERC20 Balances Loaded: {:?}", erc20_balances);
            trace!("ETH Balances Loaded: {:?}", eth_balances);
        }

        let mut shared_cache = SHARED_CACHE.write().unwrap();
        shared_cache.currencies = currencies;
        shared_cache.erc20_balance = erc20_balances;
        shared_cache.eth_balance = eth_balances;

        let (front_sender, front_receiver) = unbounded();
        let (back_sender, back_receiver) = unbounded();

        app.gui.swap_ui.front_sender = Some(front_sender.clone());
        app.gui.sender = Some(front_sender.clone());

        std::thread::spawn(move || {
            Backend::new(back_sender, front_receiver).init();
        });

        app.front_sender = Some(front_sender);
        app.back_receiver = Some(back_receiver);

        app
    }

    fn config_style(&self, ctx: &Context) -> ZeusTheme {
        let theme = ZeusTheme::new(ctx);
        let style = Style {
            visuals: theme.visuals.clone(),
            ..Style::default()
        };
        ctx.set_fonts(get_fonts());
        ctx.set_style(style);
        theme
    }

    /// Send a request to backend
    fn send_request(&mut self, request: Request) {
        if let Some(sender) = &self.front_sender {
            match sender.send(request) {
                Ok(_) => {}
                Err(e) => {
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.err_msg = ErrorMsg::new(true, e);
                }
            }
        }
    }

    fn request_eth_balance(&mut self) {
        if self.data.profile.current_wallet.is_none() {
            return;
        }

        if self.data.client().is_none() {
            return;
        }
        let chain = self.data.chain_id.id();
        let owner = self.data.wallet_address();

        let (balance_block, latest_balance) = self.data.eth_balance(chain, owner);
        let latest_block = self.data.latest_block().number;

        // balance up to date, skip
        if balance_block == latest_block {
            return;
        }

        // check if the timeout has passed
        let now = Instant::now();
        let timeout_expired =
            now.duration_since(self.last_eth_request) > Duration::from_secs(TIME_OUT);

        // timeout has not expired and chain is not ethereum, skip
        if !timeout_expired && chain != 1 {
            return;
        }

        let req = Request::EthBalance {
            owner: self.data.wallet_address(),
            chain_id: self.data.chain_id.id(),
            block: self.data.latest_block().number,
            client: self.data.client().unwrap(),
        };
        self.send_request(req);
        self.last_eth_request = now;
        // insert just the latest block to avoid duplicate requests
        // it will be overwritten when the response is received
        self.data.update_balance(chain, owner, latest_balance);
        trace!("Sent Request For ETH Balance");
    }

    /// Request the ERC20 balance of the current wallet for the SwapUI
    ///
    /// For Ethereum we only do requests on every new block
    /// For other chains their block time can vary a lot so we only do requests every 3 seconds
    fn request_erc20_balance(&mut self) {
        // no selected wallet, skip
        if self.data.wallet_address().is_zero() {
            return;
        }

        // no client, skip
        if self.data.client().is_none() {
            return;
        }

        // check if the timeout has passed
        let now = Instant::now();
        let timeout_expired =
            now.duration_since(self.last_erc20_request) > Duration::from_secs(TIME_OUT);
        let chain = self.data.chain_id.id();

        // timeout has not expired and chain is not ethereum, skip
        if !timeout_expired && chain != 1 {
            return;
        }

        // compare the latest block from oracle with the swap ui state block
        let swap_state_block;
        {
            let swap_state = SWAP_UI_STATE.read().unwrap();
            swap_state_block = swap_state.block;
        }
        let latest_block = self.data.latest_block().number;

        // if the block is the same, skip
        if swap_state_block == latest_block {
            return;
        }

        let mut swap_state = SWAP_UI_STATE.write().unwrap();
        let client = self.data.client().unwrap();

        if !swap_state.currency_in.is_native() {
            // currency is an ERC20 token
            let token = swap_state.currency_in.get_erc20().unwrap();

            let req = Request::GetERC20Balance {
                id: "input".to_string(),
                token: token.clone(),
                owner: self.data.wallet_address(),
                chain_id: self.data.chain_id.id(),
                block: latest_block,
                client: client.clone(),
            };
            self.send_request(req);
            info!("Request sent for input token: {:?}", token.symbol);
        }

        if !swap_state.currency_out.is_native() {
            let token = swap_state.currency_out.get_erc20().unwrap();

            let req = Request::GetERC20Balance {
                id: "output".to_string(),
                token: token.clone(),
                owner: self.data.wallet_address(),
                chain_id: self.data.chain_id.id(),
                block: latest_block,
                client,
            };
            self.send_request(req);
            info!("Request sent for output token: {:?}", token.symbol);
        }

        // update the last request time
        self.last_erc20_request = now;
        // update the swap state block
        swap_state.block = latest_block;
    }

    fn update_eth_balance(&mut self, balance: U256) {
        let owner = self.data.wallet_address();
        let chain_id = self.data.chain_id.id();

        // update eth balance in the shared cache
        let mut shared_cache = SHARED_CACHE.write().unwrap();
        shared_cache.eth_balance.insert(
            (chain_id, owner),
            (self.data.latest_block().number, balance),
        );
    }
}

// Main Event Loop Of The Window
// This is where we draw the UI
impl eframe::App for ZeusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(receive) = &self.back_receiver {
            match receive.try_recv() {
                Ok(response) => {
                    match response {
                        Response::EthBalance(balance) => {
                            self.update_eth_balance(balance);
                        }

                        Response::GetClient(client, chain_id) => {
                            self.data.ws_client.insert(chain_id.id(), client.clone());
                            trace!("Changed Chain: {:?}", chain_id.name().clone());

                            // setup block oracle
                            self.send_request(Request::InitOracles {
                                client,
                                chain_id: chain_id.clone(),
                            });
                        }
                    }
                }
                Err(_) => {}
            }
        }

        // this is a temp solution
        if self.data.logged_in {
            self.top_panel_h = 100.0;
            self.left_panel_w = 200.0;
        }

        if self.on_startup {
            if self.data.logged_in {
                self.send_request(Request::OnStartup {
                    chain_id: self.data.chain_id.clone(),
                    rpcs: self.data.rpc.clone(),
                });

                // run only once
                self.on_startup = false;
            }
        }

        // update to latest block
        {
            let oracle = BLOCK_ORACLE.read().unwrap();
            if self.data.latest_block().number != oracle.latest_block().number {
                self.data.latest_block = oracle.latest_block.clone();
                self.data.next_block = oracle.next_block.clone();
            }
        }

        self.request_eth_balance();
        self.request_erc20_balance();

        // Draw the UI that belongs to the Central Panel
        egui::CentralPanel::default().show(ctx, |ui| {

            // Paint the gradient mesh
            let painter = ui.painter();
            painter.add(self.gui.theme.bg_gradient.clone());

            show_login(ui, &mut self.data);


            // if we are not logged in or we are on the new profile screen, we should not paint the main UI
            if !self.data.logged_in || self.data.new_profile_screen {
                return;
            }

            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                self.gui
                    .swap_ui
                    .swap_panel(ui, &mut self.data, self.gui.theme.icons.clone());
            });
        });

        // Draw the UI that belongs to the Top Panel
        egui::TopBottomPanel::top("top_panel")
            .exact_height(self.top_panel_h)
            .show(ctx, |ui| {
                // paint the bg
                let painter = ui.painter();
                painter.add(self.gui.theme.bg_gradient.clone());

                self.gui.wallet_ui(ui, &mut self.data);

                self.gui.settings_menu(ui);
            });

        // Draw the UI that belongs to the Left Panel
        egui::SidePanel::left("left_panel")
            .exact_width(self.left_panel_w)
            .show(ctx, |ui| {
                let painter = ui.painter();
                painter.add(self.gui.theme.bg_gradient.clone());

                self.gui.select_chain(ui, &mut self.data);
                ui.add_space(10.0);
                self.gui.side_panel_menu(ui, &mut self.data);

                // Call Show methods that are not part of the main UI
                // And they depend on their own `State` or the [SHARED_UI_STATE] to be shown
                self.gui.show_network_settings_ui(ui, &mut self.data);
                show_err_msg(ui);
                tx_settings_window(ui, &mut self.data);
            });
    }
}
