use eframe::egui;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use egui::{
    vec2, Align, Align2, Button, Checkbox, Color32, FontId, Layout, RichText, TextEdit, Ui,
    Response
};

use crate::fonts::roboto_regular;
use super::icons::tx_settings_icon;
use super::misc::{frame, rich_text};
use super::ErrorMsg;
use zeus_defi::erc20::ERC20Token;
use zeus_types::app_state::{AppData, state::SHARED_UI_STATE};


use crossbeam::channel::Sender;
use alloy::primitives::Address;

use zeus_backend::types::Request;
use self::state::{SwapUIState, SWAP_UI_STATE};

pub mod state;

pub struct QuoteResult {

    /// Token price against its paired token
    pub token_price: String,

    /// The price impact of the swap
    pub price_impact: String,

    /// Selected slippage
    pub slippage: String,

    /// The real amount of tokens we will receive, after considering the pool fee and token tax if any
    pub real_amount: String,

    /// Minimum amount we may receive depending on the slippage
    pub minimum_received: String,

    /// Token Tax (If any)
    pub token_tax: String,

    /// Pool Fee
    pub pool_fee: String,

    /// Gas Cost of the swap
    pub gas_cost: String,

}


/// Manages the state of the swap UI
pub struct SwapUI {
    /// Send Request to the backend
    pub front_sender: Option<Sender<Request>>,

    pub state: Arc<RwLock<SwapUIState>>,

    /// Switch the UI on or off
    pub on: bool,

    /// Switch the output token list on or off
    pub output_token_list_on: bool,

    /// Switch the input token list on or off
    pub input_token_list_on: bool,

    /// The current input token selected
    pub input_token: ERC20Token,

    /// The current output token selected
    pub output_token: ERC20Token,

    /// The current amount of input token
    pub input_amount: String,

    /// The current amount of output token
    pub output_amount: String,

    /// The current balance of input token
    pub input_balance: String,

    /// The current balance of output token
    pub output_balance: String,

    /// The search query for a token
    pub search_token: String,

    pub input_id: String,

    pub output_id: String,

    /// A Vec of [ERC20Token] with their corresponding chain id
    pub tokens: HashMap<u64, Vec<ERC20Token>>
}

impl Default for SwapUI {
    fn default() -> Self {
        Self {
            front_sender: None,
            on: true,
            state: SWAP_UI_STATE.clone(),
            output_token_list_on: false,
            input_token_list_on: false,
            input_token: ERC20Token::eth_default_input(),
            output_token: ERC20Token::eth_default_output(),
            input_amount: "".to_string(),
            output_amount: "".to_string(),
            input_balance: "".to_string(),
            output_balance: "".to_string(),
            search_token: "".to_string(),
            input_id: String::from("input"),
            output_id: String::from("output"),
            tokens: HashMap::new()
        }
    }
}

impl SwapUI {
    /// Update input_token based on the selected chain id
    pub fn default_input(&mut self, id: u64) {
        match id {
            1 => self.input_token = ERC20Token::eth_default_input(),
            56 => self.input_token = ERC20Token::bsc_default_input(),
            8453 => self.input_token = ERC20Token::base_default_input(),
            42161 => self.input_token = ERC20Token::arbitrum_default_input(),
            _ => {}
        }
    }

    /// Update output_token based on the selected chain id
    pub fn default_output(&mut self, id: u64) {
        match id {
            1 => self.output_token = ERC20Token::eth_default_output(),
            56 => self.output_token = ERC20Token::bsc_default_output(),
            8453 => self.output_token = ERC20Token::base_default_output(),
            42161 => self.output_token = ERC20Token::arbitrum_default_output(),
            _ => {}
        }
    }

    /// Get the input or output token by an id
    fn get_token(&self, id: &str) -> ERC20Token {
        match id {
            "input" => self.input_token.clone(),
            "output" => self.output_token.clone(),
            _ => ERC20Token::eth_default_input(),
        }
    }

    /// Update the input or output token by an id
    pub fn update_token(&mut self, id: &str, token: ERC20Token) {
        match id {
            "input" => {
                self.input_token = token;
            }
            "output" => {
                self.output_token = token;
            }
            _ => {}
        }
    }

    /// Update the balance of a token by an id
    pub fn update_balance(&mut self, id: &str, balance: String) {
        match id {
            "input" => self.input_balance = balance,
            "output" => self.output_balance = balance,
            _ => {}
        }
    }

    /// Get which list is on or off by an id
    /// 
    /// `id` -> "input" or "output" token
    fn get_token_list_status(&self, id: &str) -> bool {
        match id {
            "input" => self.input_token_list_on,
            "output" => self.output_token_list_on,
            _ => false,
        }
    }

    /// Close or Open the [token_list_window] by an id
    /// 
    /// `id` -> "input" or "output" token
    /// 
    /// `on` -> true or false
    pub fn update_token_list_status(&mut self, id: &str, on: bool) {
        match id {
            "input" => {
                self.input_token_list_on = on;
            }
            "output" => {
                self.output_token_list_on = on;
            }
            _ => {}
        }
    }

    /// Send a request to the backend
    pub fn send_request(&self, request: Request) {
        if let Some(sender) = &self.front_sender {
            sender.send(request).unwrap();
        }
    }

    /// TxSettings popup
    pub fn tx_settings_window(&mut self, ui: &mut Ui, data: &mut AppData) {
        {
        let state = SHARED_UI_STATE.read().unwrap();
        if !state.tx_settings_on {
            return;
        }
    }

        egui::Window::new("Transaction Settings")
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui.set_max_size(vec2(200.0, 100.0));

                ui.vertical_centered(|ui| {
                    let priority_fee = rich_text("Priority Fee (Gwei)", 15.0);
                    let slippage_text = rich_text("Slippage", 15.0);
                    let mev_protect = rich_text("MEV Protect", 15.0);

                    let fee_field = TextEdit::singleline(&mut data.tx_settings.priority_fee)
                        .desired_width(15.0);

                    let slippage_field = TextEdit::singleline(&mut data.tx_settings.slippage)
                        .desired_width(15.0);

                    let mev_protect_check = Checkbox::new(&mut data.tx_settings.mev_protect, "");

                    ui.horizontal(|ui| {
                        ui.label(priority_fee);
                        ui.add_space(5.0);
                        ui.add(fee_field);
                       
                    });
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(slippage_text);
                        ui.add_space(5.0);
                        ui.add(slippage_field);
                    });
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(mev_protect);
                        ui.add_space(5.0);
                        ui.add(mev_protect_check);
                    });
                    ui.add_space(10.0);

                    if ui.button("Save").clicked() {
                        // TODO save the settings
                        let mut state = SHARED_UI_STATE.write().unwrap();
                        state.tx_settings_on = false;
                    }
                });
            });
                
    }

    /// Renders the swap panel
    pub fn swap_panel(&mut self, ui: &mut Ui, data: &mut AppData) {
        if !self.on {
            return;
        }

        
        let tokens = self.tokens.get(&data.chain_id.id()).unwrap_or(&vec![]).clone();
        let input_id = self.input_id.clone();
        let output_id = self.output_id.clone();

        let swap = rich_text("Swap", 20.0);
        let for_t = rich_text("For", 20.0);

        frame().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.set_width(550.0);
                ui.set_height(220.0);

                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    let response = ui.add(tx_settings_icon());
        
                    if response.clicked() {
                        let mut state = SHARED_UI_STATE.write().unwrap();
                        state.tx_settings_on = true;
                    }
                });

                
                // Input Token Field
                ui.label(swap);

                ui.horizontal(|ui| {
                    ui.add_space(115.0);
                    self.input_amount_field(ui);
                    self.token_select_button(ui, &input_id, tokens.clone(), data);
                });
                ui.add_space(10.0);

                // Output Token Field
                ui.label(for_t);

                ui.horizontal(|ui| {
                    ui.add_space(115.0);
                    self.output_amount_field(ui);
                    self.token_select_button(ui, &output_id, tokens.clone(), data);
                });

          
        });
        
        });
    }

    /// Renders the token selection list window
    fn token_list_window(&mut self, ui: &mut Ui, id: &str, tokens: Vec<ERC20Token>, data: &mut AppData) {
        
        if !self.get_token_list_status(id) {
            return;
        }

            egui::Window::new("Token List")
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .collapsible(false)
                .show(ui.ctx(), |ui| {


                ui.add(
                    TextEdit::singleline(&mut self.search_token)
                        .hint_text("Search tokens by symbol or address")
                        .min_size((200.0, 30.0).into())
                );

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (index, token) in tokens.iter().enumerate() {
                        // show tokens that match or contain the search text
                        if token.symbol.to_lowercase().contains(&self.search_token.to_lowercase()) {
                            ui.push_id(index, |ui| {
                                if
                                    ui
                                        .selectable_label(
                                            self.get_token(id) == token.clone(),
                                            token.symbol.clone()
                                        )
                                        .clicked()
                                {
                                    self.update_token(id, token.clone());

                                    // close the token list
                                    self.update_token_list_status(id, false);
                                    
                                }
                            });
                        }
                    }

                    // if search string is a valid ethereum address
                    if let Ok(address) = Address::from_str(&self.search_token) {
                        if ui.button("Add Token").clicked() {
                            println!("Adding Token: {:?}", address);
                            let client = match data.client() {
                                Some(client) => client,
                                None => {
                                    let mut state = SHARED_UI_STATE.write().unwrap();
                                    state.err_msg = ErrorMsg::new(true, "You are not connected to a node");
                                    return;
                                }
                            };
                            self.send_request(Request::GetERC20Token {
                                id: id.to_string(),
                                address,
                                client,
                                chain_id: data.chain_id.id()
                            });
                        }
                    }
                });
            });
    }

    /// Renders the token select button
    fn token_select_button(&mut self, ui: &mut Ui, id: &str, tokens: Vec<ERC20Token>, data: &mut AppData) {
        if self.token_button(id, ui).clicked() {
            self.update_token_list_status(id, true);
        }
        self.token_list_window(ui, id, tokens, data);
    }

    /// Render the balance of the token
    fn token_balance(&mut self, ui: &mut Ui) {
        let balance = RichText::new("Balance:")
            .size(15.0)
            .family(roboto_regular())
            .color(Color32::WHITE);

        ui.label(balance);
        ui.add_space(5.0);
        ui.label(self.input_balance.clone());
    }

    /// Creates the field for the input amount
    fn input_amount_field(&mut self, ui: &mut Ui) {
        let font = FontId { size: 23.0, family: roboto_regular() };

        let field = TextEdit::singleline(&mut self.input_amount)
            .font(font.clone())
            .min_size((100.0, 30.0).into())
            .hint_text(
                RichText::new("0")
                    .color(Color32::from_gray(128))
                    .size(23.0)
                    .family(roboto_regular())
            );

        ui.add(field);
    }

    /// Creates the field for the output amount
    fn output_amount_field(&mut self, ui: &mut Ui) {
        let font = FontId { size: 23.0, family: roboto_regular() };

        let field = TextEdit::singleline(&mut self.output_amount)
            .font(font.clone())
            .min_size((100.0, 30.0).into())
            .hint_text(
                RichText::new("0")
                    .color(Color32::from_gray(128))
                    .size(23.0)
                    .family(roboto_regular())
            );

        ui.add(field);
    }

    /// Create the token button
    fn token_button(&mut self, id: &str, ui: &mut Ui) -> Response {
        let token_symbol = RichText::new(self.get_token(id).symbol.clone())
            .size(15.0)
            .family(roboto_regular())
            .color(Color32::WHITE);

       let button = Button::new(token_symbol).min_size(vec2(30.0, 15.0));
       let res = ui.add(button);
       res
    }
}
