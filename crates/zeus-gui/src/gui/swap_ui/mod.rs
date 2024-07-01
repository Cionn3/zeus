use eframe::egui::{self, SelectableLabel};
use std::str::FromStr;
use std::sync::{ Arc, RwLock };
use egui::{
    vec2,
    Align,
    Align2,
    Button,
    Color32,
    FontId,
    Layout,
    RichText,
    TextEdit,
    Ui,
    Response
};

use crate::fonts::roboto_regular;
use super::{icons::tx_settings_icon, misc::{ frame, rich_text }, ErrorMsg};
use zeus_types::app_state::{ AppData, state::SHARED_UI_STATE };
use zeus_types::defi::{currency::Currency, utils::{parse_wei, format_wei}};
use zeus_backend::types::Request;
use zeus_types::app_state::state::{ SelectedCurrency, SwapUIState, SWAP_UI_STATE };
use zeus_backend::types::SwapParams;

use crossbeam::channel::Sender;
use alloy::primitives::{Address, U256};

use tracing::{info, error};


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
        let currencies;
        
        {
            let shared = SHARED_UI_STATE.read().unwrap();
            if !shared.swap_ui_on {
                return;
            }
            
            let state = SWAP_UI_STATE.read().unwrap();

            currencies = state.currencies.get(&data.chain_id.id()).unwrap_or(&vec![]).clone();
        }

        let swap = rich_text("Swap", 20.0);
        let for_t = rich_text("For", 20.0);

        frame().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                let ui_width = 550.0;
                let ui_height = 220.0;
                ui.set_width(ui_width);
                ui.set_height(ui_height);

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
                        self.token_select_button(ui, "input", currencies.clone(), data);
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
                        self.token_select_button(ui, "output", currencies.clone(), data);
                        self.token_balance(ui, "output");
                    });
                });

                ui.horizontal(|ui| {
                    ui.add_space(160.0);
                    self.get_quote_button(ui, data);
                    ui.add_space(10.0);
                    self.swap_button(ui, data);
                });
                ui.add_space(20.0);
                self.quote_result(ui, data);
            }); // vertical centered main frame
        }); // frame
    }

    fn quote_result(&self, ui: &mut Ui, data: &mut AppData) {
        // Quote Result

        let state = SWAP_UI_STATE.read().unwrap();
        let quote_result = state.quote_result.clone();

        ui.horizontal(|ui| {
            ui.add_space(170.0);

            frame().show(ui, |ui| {
                ui.set_width(300.0);
                ui.set_height(150.0);

                ui.vertical_centered(|ui| {
                    // Block
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.label(rich_text("Block:", 12.0)).on_hover_text(
                                "Quote result is based on this block"
                            );
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(quote_result.block_number.to_string());
                        });
                    });

                    // Slippage
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.label(rich_text("Slippage:", 12.0));
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(quote_result.slippage.clone());
                        });
                    });

                    // Expected Amount
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.label(rich_text("Expected Amount:", 12.0)).on_hover_text(
                                "Expected amount of tokens to be received considering the pool fee and token tax (if any)"
                            );
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(quote_result.output_token_amount());
                        });
                    });

                    // Minimum Received
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.label(rich_text("Minimum Received:", 12.0)).on_hover_text(
                                "This is the minimum amount you may receive based on your slippage, You cannot receive less than this amount otherwise the transaction will revert"
                            );
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(quote_result.minimum_received_amount());
                        });
                    });
                });
            });
        }); // frame
    }

    /// Renders the token selection list window
    fn token_list_window(
        &mut self,
        ui: &mut Ui,
        id: &str,
        currencies: Vec<Currency>,
        data: &mut AppData
    ) {
        {
            let state = SWAP_UI_STATE.read().unwrap();
            if !state.get_token_list_status(id) {
                return;
            }
        }

        egui::Window
            ::new("Token List")
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                // ! Lock here maybe held too much
                let mut state = SWAP_UI_STATE.write().unwrap();

                ui.add(
                    TextEdit::singleline(&mut state.search_token)
                        .hint_text("Search tokens by symbol or address")
                        .min_size((200.0, 30.0).into())
                );

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (index, currency) in currencies.iter().enumerate() {
                        // show currencies/tokens that match or contain the search text

                        match currency {
                            // Currency is an ERC20 token
                            Currency::ERC20(token) => {
                                if token.symbol.to_lowercase().contains(&state.search_token.to_lowercase()) {
                                    ui.push_id(index, |ui| {

                                        // bool check, if true the token is selected
                                        let selected_token = state.get_token(id).currency.symbol() == token.symbol.clone();


                                        let selectable_label = SelectableLabel::new(selected_token, token.symbol.clone());
                                        let res = ui.add(selectable_label);

                                        if res.clicked() {
                                            state.replace_token(id, SelectedCurrency::new_from_erc(token.clone(), U256::ZERO));
                                            state.update_token_list_status(id, false); 
                                        }

                                    });
                                }
                            }
                            // Currency is a native token
                            Currency::Native(native) => {
                                if native.symbol.to_lowercase().contains(&state.search_token.to_lowercase()) {
                                    ui.push_id(index, |ui| {

                                        let selected_currency = state.get_token(id).currency.symbol() == native.symbol.clone();

                                        let selectable_label = SelectableLabel::new(selected_currency, native.symbol.clone());
                                        let res = ui.add(selectable_label);
                                        
                                        if res.clicked() {
                                            state.replace_token(id, SelectedCurrency::new_from_native(native.clone()));
                                            state.update_token_list_status(id, false);
                                        }
                                    });
                                }
                            }
                        };
                    }

                    // if search string is a valid ethereum address
                    if let Ok(address) = Address::from_str(&state.search_token) {
                        if ui.button("Add Token").clicked() {
                            println!("Adding Token: {:?}", address);
                            let client = match data.client() {
                                Some(client) => client,
                                None => {
                                    let mut state = SHARED_UI_STATE.write().unwrap();
                                    state.err_msg = ErrorMsg::new(
                                        true,
                                        "You are not connected to a node"
                                    );
                                    return;
                                }
                            };
                            self.send_request(Request::GetERC20Token {
                                id: id.to_string(),
                                owner: data.wallet_address(),
                                address,
                                client,
                                chain_id: data.chain_id.id(),
                            });
                        }
                    }
                    

                });
            });
    }

    /// Renders the token select button
    fn token_select_button(
        &mut self,
        ui: &mut Ui,
        id: &str,
        currencies: Vec<Currency>,
        data: &mut AppData
    ) {
        if self.token_button(id, ui).clicked() {
            let mut state = SWAP_UI_STATE.write().unwrap();
            state.update_token_list_status(id, true);
        }
        self.token_list_window(ui, id, currencies, data);
    }

    /// Render the balance of the token
    fn token_balance(&mut self, ui: &mut Ui, id: &str) {
        let currency;
        {
            let state = SWAP_UI_STATE.read().unwrap();
            currency = state.get_token(id).clone();
        }

        let balance_text = RichText::new("Balance:")
            .size(12.0)
            .family(roboto_regular())
            .color(Color32::WHITE);

        let balance = format_wei(&currency.balance, currency.decimals());
        let formated_balance = format!("{:.4}", balance);

        ui.horizontal(|ui| {
            ui.label(balance_text);
            ui.add_space(1.0);
            ui.label(formated_balance);
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
        let token_symbol = RichText::new(state.get_token(id).currency.symbol().clone())
            .size(15.0)
            .family(roboto_regular())
            .color(Color32::WHITE);

        let button = Button::new(token_symbol)
            .min_size(vec2(30.0, 15.0))
            .rounding(10.0)
            .stroke((0.3, Color32::WHITE));
        let res = ui.add(button);
        res
    }

    /// Swap Button
    fn swap_button(&mut self, ui: &mut Ui, data: &mut AppData) {
        let text = RichText::new("Swap").size(15.0).family(roboto_regular()).color(Color32::WHITE);
        let button = Button::new(text)
            .min_size(vec2(100.0, 30.0))
            .rounding(10.0)
            .stroke((0.3, Color32::WHITE));
        if ui.add(button).clicked() {
            // TODO
            println!("Swapping...");
        }
    }

    /// Get Quote Button
    fn get_quote_button(&mut self, ui: &mut Ui, data: &mut AppData) {
        let text = RichText::new("Get Quote")
            .size(15.0)
            .family(roboto_regular())
            .color(Color32::WHITE);
        let button = Button::new(text)
            .min_size(vec2(100.0, 30.0))
            .rounding(10.0)
            .stroke((0.3, Color32::WHITE));
        if ui.add(button).clicked() {
            if data.client().is_none() {
                let mut state = SHARED_UI_STATE.write().unwrap();
                state.err_msg = ErrorMsg::new(true, "You are not connected to a node");
                return;
            }
            let swap_state = SWAP_UI_STATE.read().unwrap();

            // TODO
            /* 
            self.send_request(Request::GetQuoteResult {
                params: SwapParams {
                    token_in: swap_state.input_token.clone(),
                    token_out: swap_state.output_token.clone(),
                    amount_in: swap_state.input_token.amount_to_swap.clone(),
                    slippage: data.tx_settings.slippage.clone(),
                    chain_id: data.chain_id.clone(),
                    block: data.block_info.0.full_block.clone().unwrap(),
                    client: data.client().unwrap().clone(),
                    caller: data.wallet_address(),
                },
            });*/
        }
    }
}
