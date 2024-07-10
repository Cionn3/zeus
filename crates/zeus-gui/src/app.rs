use std::{ collections::HashMap, time::{ Instant, Duration }, sync::Arc };
use eframe::{ egui, CreationContext };
use egui::{ vec2, Align2, ComboBox, Context, Style, Ui };

use crossbeam::channel::{ unbounded, Receiver, Sender };


use crate::{
    fonts::get_fonts,
    gui::{
        ZeusTheme,
        GUI,
        misc::{ login_screen, new_profile_screen, tx_settings_window, rich_text, frame },
        icons::IconTextures,
    },
};

use zeus_backend::{ db::ZeusDB, types::{ Request, Response }, Backend };
use zeus_chain::{defi_types::currency::Currency, BLOCK_ORACLE, alloy::primitives::U256};
use zeus_shared_types::{AppData, ErrorMsg, SHARED_UI_STATE, SWAP_UI_STATE};

use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    prelude::*,
    util::SubscriberInitExt,
    EnvFilter,
};

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::Registry;
use tracing::{error, info, trace};



/// The timeout for requesting eth and erc20 balances
/// This do not apply for Ethereum since it has a block time of 12 secs and cannot cause a lot of rpc calls
const TIME_OUT: u64 = 3;

/// The main application struct
pub struct ZeusApp {
    /// The GUI components of the application
    pub gui: GUI,

    /// The icons used in the application
    pub icons: Arc<IconTextures>,

    /// Send Data to backend
    pub front_sender: Option<Sender<Request>>,

    /// Receive Data from backend
    pub back_receiver: Option<Receiver<Response>>,

    /// The app data of the application
    pub data: AppData,

    pub last_eth_request: Instant,

    pub last_erc20_request: Instant,

    pub last_quote_request: Instant,
}


fn setup_logging() -> (WorkerGuard, WorkerGuard) {
    // Setup for file appenders
    let trace_appender = tracing_appender::rolling::daily("./logs", "trace.log");
    let output_appender = tracing_appender::rolling::daily("./logs", "output.log");

    // Creating non-blocking writers
    let (trace_writer, trace_guard) = tracing_appender::non_blocking(trace_appender);
    let (output_writer, output_guard) = tracing_appender::non_blocking(output_appender);

    // Defining filters
    let console_filter = EnvFilter::new("zeus,zeus_core,zeus_chain,zeus_backend=info,error,warn");
    let trace_filter = EnvFilter::new("zeus,zeus_core,zeus_chain_zeus_backend=trace");
    let output_filter = EnvFilter::new("zeus,zeus_core,zeus_chain_zeus_backend=info,error,warn");

    // Setting up layers
    let console_layer = fmt::layer().with_writer(std::io::stdout).with_filter(console_filter);
    let trace_layer = fmt::layer().with_writer(trace_writer).with_filter(trace_filter);
    let output_layer = fmt::layer().with_writer(output_writer).with_filter(output_filter);

    // Applying configuration
    Registry::default().with(trace_layer).with(console_layer).with(output_layer).init();

    (trace_guard, output_guard)
}

impl ZeusApp {
    pub fn new(cc: &CreationContext) -> Self {
        let _guard = setup_logging();

        let icons = Arc::new(IconTextures::new(&cc.egui_ctx).unwrap());
        let gui = GUI::new_default(icons.clone());

        let mut app = Self {
            gui,
            icons,
            front_sender: None,
            back_receiver: None,
            data: AppData::default(),
            last_eth_request: Instant::now(),
            last_erc20_request: Instant::now(),
            last_quote_request: Instant::now(),
        };   

        app.config_style(&cc.egui_ctx);

        match app.data.load_rpc() {
            Ok(_) => {}
            Err(e) => {
                println!("Error Loading rpc.json: {}", e);
            }
        }

        let currencies: HashMap<u64, Vec<Currency>>;

        {
            let zeus_db = ZeusDB::new().unwrap();

            match zeus_db.insert_default() {
                Ok(_) => {}
                Err(e) => {
                    error!("Error Inserting Default Tokens: {}", e);
                }
            }

            currencies = match zeus_db.load_currencies(app.data.supported_networks()) {
                Ok(currencies) => currencies,
                Err(e) => {
                    error!("Error Loading Currencies: {}", e);
                    HashMap::new()
                }
            };
        }

        let mut swap_ui_state = SWAP_UI_STATE.write().unwrap();
        swap_ui_state.currencies = currencies;

        let (front_sender, front_receiver) = unbounded();
        let (back_sender, back_receiver) = unbounded();

        app.gui.swap_ui.front_sender = Some(front_sender.clone());
        app.gui.sender = Some(front_sender.clone());

        std::thread::spawn(move || {
            Backend::new(back_sender, front_receiver).init();
        });

        app.front_sender = Some(front_sender);
        app.back_receiver = Some(back_receiver);

        if app.data.profile_exists {
            app.send_request(Request::OnStartup {
                chain_id: app.data.chain_id.clone(),
                rpcs: app.data.rpc.clone(),
            });
        }


        app
    }

    fn config_style(&self, ctx: &Context) {
        let style = Style {
            visuals: ZeusTheme::default().visuals,
            ..Style::default()
        };
        ctx.set_fonts(get_fonts());
        ctx.set_style(style);
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
        if self.data.wallet_address().is_zero() {
            return;
        }
        
        if !self.data.should_update_balance() {
            return;
        }

        if let Some(client) = self.data.client() {
            let now = Instant::now();
                    if now.duration_since(self.last_eth_request) > Duration::from_secs(TIME_OUT) {
                        let req = Request::EthBalance {
                            address: self.data.wallet_address(),
                            client,
                        };
                        self.send_request(req);
                        self.last_eth_request = now;
                    }
                }         
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
        let timeout_expired = now.duration_since(self.last_erc20_request) > Duration::from_secs(TIME_OUT);
        let chain = self.data.chain_id.id();

        // timeout has not expired and chain is not ethereum, skip
        if !timeout_expired && chain != 1{
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

        if !swap_state.currency_in.is_native() {
            // currency is an ERC20 token
            let token = swap_state.currency_in.get_erc20().unwrap();
          
            let req = Request::GetERC20Balance {
                id: "input".to_string(),
                token: token.clone(),
                owner: self.data.wallet_address(),
                chain_id: self.data.chain_id.id(),
                block: latest_block,
                client: self.data.ws_client.get(&self.data.chain_id.id()).unwrap().clone(),
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
                client: self.data.ws_client.get(&self.data.chain_id.id()).unwrap().clone(),
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
        if let Some(wallet) = &mut self.data.profile.current_wallet {
            let block = self.data.block_info.0.number;
            let chain_id = self.data.chain_id.id();
            wallet.update_balance(chain_id, balance, block);

            // * update the balance in the swap ui if a native currency is selected
            let mut swap_state = SWAP_UI_STATE.write().unwrap();
            if swap_state.currency_in.is_native() {
                swap_state.currency_in.balance = balance.to_string();
            }

            if swap_state.currency_out.is_native() {
                swap_state.currency_out.balance = balance.to_string();
            }
        }
    }



    fn draw_login(&mut self, ui: &mut Ui) {
        if self.data.profile_exists && !self.data.logged_in {
            login_screen(ui, self);
        }

        if self.data.new_profile_screen {
            new_profile_screen(ui, self);
        }
    }

    fn select_chain(&mut self, ui: &mut Ui) {
        let chain_ids = self.data.chain_ids.clone();
        ui.horizontal(|ui| {
            ui.add(self.icons.chain_icon(self.data.chain_id.id()));

            ComboBox::from_label("")
                .selected_text(self.data.chain_id.name())
                .show_ui(ui, |ui| {
                    for chain_id in chain_ids {
                        if ui.selectable_value(&mut self.data.chain_id, chain_id.clone(), chain_id.name()).clicked() {
                            self.send_request(Request::GetClient {
                                chain_id: chain_id.clone(),
                                rpcs: self.data.rpc.clone(),
                                clients: self.data.ws_client.clone()
                            });
                            let mut swap_ui_state = SWAP_UI_STATE.write().unwrap();
                            swap_ui_state.default_input(chain_id.id());
                            swap_ui_state.default_output(chain_id.id());                
                        }
                    }
                });
                ui.add(self.icons.connected_icon(self.data.connected(self.data.chain_id.id())));

        });
    }

    /// Show an error message if needed
    fn err_msg(&mut self, ui: &mut Ui) {
        let err_msg;
        {
            let state = SHARED_UI_STATE.read().unwrap();
            err_msg = state.err_msg.msg.clone();
            if !state.err_msg.on {
                return;
            }
        }

        egui::Window
            ::new("Error")
            .resizable(false)
            .anchor(Align2::CENTER_TOP, vec2(0.0, 0.0))
            .collapsible(false)
            .title_bar(false)
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    
                    let msg_text = rich_text(&err_msg, 16.0);
                    let close_text = rich_text("Close", 16.0);

                    ui.label(msg_text);
                    ui.add_space(5.0);
                    if ui.button(close_text).clicked() {
                        let mut state = SHARED_UI_STATE.write().unwrap();
                        state.err_msg.on = false;
                    }
                });
            });
    }

    // TODO: Auto close it after a few seconds
    /// Show an info message if needed
    fn info_msg(&mut self, ui: &mut Ui) {
        {
            let state = SHARED_UI_STATE.read().unwrap();
            if !state.info_msg.on {
                return;
            }
        }

        ui.vertical_centered_justified(|ui| {
            frame().show(ui, |ui| {
                ui.set_max_size(vec2(1000.0, 50.0));

                let state = SHARED_UI_STATE.read().unwrap();
                let msg_text = rich_text(&state.info_msg.msg, 16.0);
                let close_text = rich_text("Close", 16.0);

                ui.label(msg_text);
                ui.add_space(5.0);
                if ui.button(close_text).clicked() {
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.info_msg.on = false;
                }
            });
        });
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
                            info!("Received client for chain: {:?}", chain_id);
                            self.data.ws_client.insert(chain_id.id(), client.clone());
                            info!("Changed Chain: {:?}", chain_id.name().clone());
                            
                            // setup block oracle
                            self.send_request(Request::InitOracles {
                                client,
                                chain_id: chain_id.clone(),
                            });
                            info!("Request sent to init oracles for chain: {:?}", chain_id);
                        }

                    }
                }
                Err(_) => {}
            }
        }

        // update to latest block
        {
            let oracle = BLOCK_ORACLE.read().unwrap();
            self.data.block_info = oracle.get_block_info();
        }

        self.request_eth_balance();
        self.request_erc20_balance();
        

        // Draw the UI that belongs to the Top Panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| { 
            self.gui.render_wallet_ui(ui, &mut self.data);
            // self.info_msg(ui);
        });

        // Draw the UI that belongs to the Central Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                self.err_msg(ui);
                self.draw_login(ui);
            });

             // if we are not logged in or we are on the new profile screen, we should not draw the main UI
            if !self.data.logged_in || self.data.new_profile_screen {
                return;
            } 
            

            ui.vertical_centered_justified(|ui| {
                ui.add_space(100.0);
                self.gui.swap_ui.swap_panel(ui, &mut self.data, self.icons.clone());
                self.gui.networks_ui(ui, &mut self.data);
                tx_settings_window(ui, &mut self.data);
            });
        });

        // Draw the UI that belongs to the Left Panel
        egui::SidePanel::left("left_panel")
            .exact_width(170.0)
            .show(ctx, |ui| {
                self.select_chain(ui);
                ui.add_space(10.0);
                self.gui.menu(ui, &mut self.data);
            });
    }
}