use eframe::egui;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use egui::{
    vec2, Align, Align2, Button, Color32, FontId, Layout, RichText, TextEdit, Ui,
    Response
};

use crate::fonts::roboto_regular;
use super::icons::tx_settings_icon;
use super::misc::{frame, rich_text};
use super::ErrorMsg;
use zeus_types::defi::erc20::ERC20Token;
use zeus_types::app_state::{AppData, state::SHARED_UI_STATE};


use crossbeam::channel::Sender;
use alloy::primitives::Address;

use zeus_backend::types::Request;
use zeus_types::app_state::state::{SelectedToken, SwapUIState, SWAP_UI_STATE};





/// Manages the state of the swap UI
pub struct SwapUI {
    /// Send Request to the backend
    pub front_sender: Option<Sender<Request>>,

    pub state: Arc<RwLock<SwapUIState>>,

}

impl Default for SwapUI {
    fn default() -> Self {
        Self {
            front_sender: None,
            state: SWAP_UI_STATE.clone(),
        }
    }
}

impl SwapUI {

    /// Send a request to the backend
    pub fn send_request(&self, request: Request) {
        if let Some(sender) = &self.front_sender {
            sender.send(request).unwrap();
        }
    }

/// Renders the swap panel
pub fn swap_panel(&mut self, ui: &mut Ui, data: &mut AppData) {
    let tokens;
    {
        let state = SWAP_UI_STATE.read().unwrap();
        let shared = SHARED_UI_STATE.read().unwrap();
        tokens = state.tokens.get(&data.chain_id.id()).unwrap_or(&vec![]).clone();
        if !shared.swap_ui_on {
            return;
        }
    }

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
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    self.token_select_button(ui, "input", tokens.clone(), data);
                    self.token_balance(ui, "input");
                });
            });
            ui.add_space(10.0);

            // Output Token Field
            ui.label(for_t);

            ui.horizontal(|ui| {
                ui.add_space(115.0);
                self.output_amount_field(ui);
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    self.token_select_button(ui, "output", tokens.clone(), data);
                    self.token_balance(ui, "output");
                });
            });

            // Quote Result
            
            let state = SWAP_UI_STATE.read().unwrap();
            let quote_result = state.quote_result.clone();
            
            let real_amount_txt = rich_text("Real Amount", 15.0);
            ui.horizontal(|ui| {
                ui.label(real_amount_txt);
                ui.add_space(10.0);
                ui.label(state.output_token.token.readable(quote_result.real_amount));
            });

            let minimum_received_txt = rich_text("Minimum Received", 15.0);
            ui.horizontal(|ui| {
                ui.label(minimum_received_txt);
                ui.add_space(10.0);
                ui.label(state.output_token.token.readable(quote_result.minimum_received));
            });
        });
    });
}


    /// Renders the token selection list window
    fn token_list_window(&mut self, ui: &mut Ui, id: &str, tokens: Vec<ERC20Token>, data: &mut AppData) {
        
        {
            let state = SWAP_UI_STATE.read().unwrap();
            if !state.get_token_list_status(id) {
                return;
            }
        }

            egui::Window::new("Token List")
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .collapsible(false)
                .show(ui.ctx(), |ui| {

                    let mut state = SWAP_UI_STATE.write().unwrap();

                ui.add(
                    TextEdit::singleline(&mut state.search_token)
                        .hint_text("Search tokens by symbol or address")
                        .min_size((200.0, 30.0).into())
                );

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (index, token) in tokens.iter().enumerate() {
                        // show tokens that match or contain the search text
                        if token.symbol.to_lowercase().contains(&state.search_token.to_lowercase()) {
                            ui.push_id(index, |ui| {
                                if
                                    ui
                                        .selectable_label(
                                            state.get_token(id).token == token.clone(),
                                            token.symbol.clone()
                                        )
                                        .clicked()
                                {
                                    state.replace_token(id, SelectedToken::new(token.clone()));

                                    // close the token list
                                    state.update_token_list_status(id, false);
                                    
                                }
                            });
                        }
                    }

                    // if search string is a valid ethereum address
                    if let Ok(address) = Address::from_str(&state.search_token) {
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
            let mut state = SWAP_UI_STATE.write().unwrap();
            state.update_token_list_status(id, true);
        }
        self.token_list_window(ui, id, tokens, data);
    }

    /// Render the balance of the token
    fn token_balance(&mut self, ui: &mut Ui, id: &str) {
        let token;
        {
        let state = SWAP_UI_STATE.read().unwrap();
        token = state.get_token(id).clone();
        }

        let balance_text = RichText::new("Balance:")
            .size(12.0)
            .family(roboto_regular())
            .color(Color32::WHITE);

        ui.horizontal(|ui| {
            ui.label(balance_text);
            ui.add_space(1.0);
            ui.label(token.token.readable(token.balance));
        });
    }
    
    

    /// Creates the field for the input amount
    fn input_amount_field(&mut self, ui: &mut Ui) {
        let font = FontId { size: 23.0, family: roboto_regular() };
        let mut state = SWAP_UI_STATE.write().unwrap();

        let field = TextEdit::singleline(&mut state.input_token.amount_to_swap)
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
        let mut state = SWAP_UI_STATE.write().unwrap();

        let field = TextEdit::singleline(&mut state.output_token.amount_to_swap)
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
        let state = SWAP_UI_STATE.read().unwrap();
        let token_symbol = RichText::new(state.get_token(id).token.symbol.clone())
            .size(15.0)
            .family(roboto_regular())
            .color(Color32::WHITE);

       let button = Button::new(token_symbol).min_size(vec2(30.0, 15.0)).rounding(10.0).stroke((0.3, Color32::WHITE));
       let res = ui.add(button);
       res
    }
}
