use std::{ collections::HashMap, time::{ Instant, Duration } };
use eframe::{ egui, CreationContext };
use egui::{ vec2, Align2, ComboBox, Context, Style, Ui };

use crossbeam::channel::{ unbounded, Receiver, Sender };
use zeus_types::defi::erc20::ERC20Token;
use alloy::primitives::U256;
use zeus_types::app_state::{AppData, state::*};
use zeus_utils::parse_ether;

use crate::{
    fonts::get_fonts,
    gui::{
        ZeusTheme,
        GUI,
        misc::{ login_screen, new_profile_screen, tx_settings_window, rich_text, frame },
        icons::*,
    },
};

use zeus_backend::{ db::ZeusDB, types::{ Request, Response, SwapParams }, Backend };


pub mod gui;
pub mod fonts;

/// Rate Limit
const TIME_OUT: u64 = 2;

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
}

impl Default for ZeusApp {
    fn default() -> Self {
        Self {
            gui: GUI::default(),
            front_sender: None,
            back_receiver: None,
            data: AppData::default(),
            last_eth_request: Instant::now(),
            last_erc20_request: Instant::now(),
            last_quote_request: Instant::now(),
        }
    }
}

impl ZeusApp {
    pub fn new(cc: &CreationContext) -> Self {
        let mut app = Self::default();
        app.config_style(&cc.egui_ctx);

        match app.data.load_rpc() {
            Ok(_) => {}
            Err(e) => {
                println!("Error Loading rpc.json: {}", e);
            }
        }

        let tokens: HashMap<u64, Vec<ERC20Token>>;

        {
            let zeus_db = ZeusDB::new().unwrap();

            match zeus_db.insert_default() {
                Ok(_) => {}
                Err(e) => {
                    println!("Error Inserting Default Tokens: {}", e);
                }
            }

            tokens = match zeus_db.load_tokens(app.data.supported_networks()) {
                Ok(tokens) => tokens,
                Err(e) => {
                    println!("Error Loading Tokens: {}", e);
                    HashMap::new()
                }
            };
        }

        let mut swap_ui_state = SWAP_UI_STATE.write().unwrap();
        swap_ui_state.tokens = tokens;

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

        let _state = SHARED_UI_STATE.write().unwrap();

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

    fn update_block(&mut self) {
        self.send_request(Request::GetBlockInfo);

    }

    fn request_quote_result(&mut self) {
        let swap_state = SWAP_UI_STATE.read().unwrap();

        if swap_state.input_token.amount_to_swap.is_empty() {
            return;
        }

        let time = Instant::now();
        if time.duration_since(self.last_quote_request) > Duration::from_secs(TIME_OUT) {
            let amount_in = parse_ether(&swap_state.input_token.amount_to_swap).unwrap();
        self.send_request(Request::GetQuoteResult {
            params: SwapParams {
                token_in: swap_state.input_token.token.clone(),
                token_out: swap_state.output_token.token.clone(),
                amount_in: swap_state.input_token.amount_to_swap.clone(),
                slippage: self.data.tx_settings.slippage.clone(),
                chain_id: self.data.chain_id.clone(),
                block: self.data.block_info.0.full_block.clone().unwrap(),
                client: self.data.ws_client.get(&self.data.chain_id.id()).unwrap().clone(),
                caller: self.data.wallet_address(),
                },
            });
            self.last_quote_request = time;
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

    fn request_erc20_balance(&mut self) {
        if self.data.wallet_address().is_zero() {
            return;
        }

        if !self.data.ws_client.contains_key(&self.data.chain_id.id()) {
            return;
        }


        let now = Instant::now();
        if now.duration_since(self.last_erc20_request) > Duration::from_secs(TIME_OUT) {
        let swap_state = SWAP_UI_STATE.read().unwrap();
        let latest_block = self.data.block_info.0.number;
        if latest_block > swap_state.block {
            let req = Request::GetERC20Balance {
                id: "input".to_string(),
                token: swap_state.input_token.token.clone(),
                owner: self.data.wallet_address(),
                chain_id: self.data.chain_id.id(),
                block: latest_block,
                client: self.data.ws_client.get(&self.data.chain_id.id()).unwrap().clone(),
            };
            self.send_request(req);

            let req = Request::GetERC20Balance {
                id: "output".to_string(),
                token: swap_state.output_token.token.clone(),
                owner: self.data.wallet_address(),
                chain_id: self.data.chain_id.id(),
                block: latest_block,
                client: self.data.ws_client.get(&self.data.chain_id.id()).unwrap().clone(),
            };

            self.send_request(req);
            self.last_erc20_request = now;
        }
        
    }
        
    }

    fn update_eth_balance(&mut self, balance: U256) {
        if let Some(wallet) = &mut self.data.profile.current_wallet {
            let block = self.data.block_info.0.number;
            let chain_id = self.data.chain_id.id();
            wallet.update_balance(chain_id, balance, block);
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
        let networks = self.data.networks.clone();
        ui.horizontal(|ui| {
            ui.add(get_chain_icon(self.data.chain_id.id()));

            ComboBox::from_label("")
                .selected_text(self.data.chain_id.name())
                .show_ui(ui, |ui| {
                    for id in networks.iter().map(|chain_id| chain_id.clone()) {
                        if
                            ui
                                .selectable_value(&mut self.data.chain_id, id.clone(), id.name())
                                .clicked()
                        {
                            println!("Selected Chain: {:?}", id);
                            self.send_request(Request::GetClient {
                                chain_id: id.clone(),
                                rpcs: self.data.rpc.clone(),
                                clients: self.data.ws_client.clone()
                            });
                            let mut swap_ui_state = SWAP_UI_STATE.write().unwrap();
                            swap_ui_state.default_input(id.id());
                            swap_ui_state.default_output(id.id());
                        }
                    }
                });
            ui.add(connected_icon(self.data.connected(self.data.chain_id.id())));
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
                        Response::GetBlockInfo(block_info) => {
                            self.data.block_info = block_info;
                        }

                        Response::GetQuoteResult(result) => {
                            println!("Swap Response: {:?}", result);
                        }

                        Response::EthBalance(balance) => {
                            self.update_eth_balance(balance);
                        }

                        Response::GetClient(client, chain_id) => {
                            self.data.ws_client.insert(chain_id.id(), client.clone());
                            // setup oracles
                            println!("Changed Chain: {:?}", chain_id.name().clone());
                            println!("Sending request to init oracles");
                            self.send_request(Request::InitOracles {
                                client,
                                chain_id,
                            });
                        }

                    }
                }
                Err(_) => {}
            }
        }

        self.update_block();
        self.request_eth_balance();
        self.request_erc20_balance();
        self.request_quote_result();

        // Draw the UI that belongs to the Top Panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                self.gui.render_wallet_ui(ui, &mut self.data);
                self.info_msg(ui);
            });
        });

        // Draw the UI that belongs to the Central Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                self.err_msg(ui);
                self.draw_login(ui);
            });

            
            if !self.data.logged_in || self.data.new_profile_screen {
                return;
            } 
            

            ui.vertical_centered_justified(|ui| {
                ui.add_space(100.0);
                self.gui.swap_ui.swap_panel(ui, &mut self.data);
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
