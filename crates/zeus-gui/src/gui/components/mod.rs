pub mod send_crypto_screen;
pub mod swap_ui;

use crate::fonts::roboto_regular;
use eframe::egui::{vec2, Align2, Button, Color32, emath::Vec2b, RichText, ScrollArea, TextEdit, Ui, Sense, Window};
use std::str::FromStr;
use tracing::trace;
use zeus_chain::{alloy::primitives::Address, defi_types::currency::Currency, utils::format_wei};
use zeus_backend::types::Request;
use zeus_shared_types::{cache::SHARED_CACHE, AppData, UiState, SHARED_UI_STATE};
use crossbeam::channel::Sender;

pub struct TokenSelectionWindow {

    pub state: UiState,

    pub search_query: String,

    pub front_sender: Option<Sender<Request>>
}

impl TokenSelectionWindow {
    pub fn new() -> Self {
        Self {
            state: UiState::default(),
            search_query: String::new(),
            front_sender: None,
        }
    }

    pub fn open(&mut self) {
        self.state = UiState::OPEN;
    }

    pub fn close(&mut self) {
        self.state = UiState::CLOSE;
    }

    /// Send a request to the backend
    pub fn send_request(&self, request: Request) {
        if let Some(sender) = &self.front_sender {
            sender.send(request).unwrap();
        }
    }

    /// This shows the token selection Window
    /// 
    /// # Arguments
    /// 
    /// * `ui` - The egui Ui
    /// * `data` - The AppData
    /// * `currencies` - The list of currencies
    /// * `id` - The id of the token, This is actually used to identify the input or output token if we call this from the SwapUI otherwise
    /// just pass an empty string
    /// 
    /// # Returns
    /// 
    /// The selected currency
    fn show(
        &mut self,
        ui: &mut Ui,
        data: &AppData,
        currencies: Vec<Currency>,
        id: &str,
    ) -> Option<Currency> {
        if self.state.is_close() {
            return None;
        }

        let chain_id = data.chain_id.id();
        let owner = data.wallet_address();

        let select = RichText::new("Select a Token")
            .family(roboto_regular())
            .size(18.0)
            .color(Color32::WHITE);

        let mut selected_currency: Option<Currency> = None;

        Window::new(select)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .resizable(false)
            .collapsible(false)
            .fade_in(true)
            .fade_out(true)
            .show(ui.ctx(), |ui| {
                ui.set_min_size(vec2(200.0, 130.0));

                ui.vertical_centered(|ui| {
                    ui.add(
                        TextEdit::singleline(&mut self.search_query)
                            .hint_text("Search tokens by symbol or address")
                            .min_size((200.0, 30.0).into()),
                    );
                    ui.add_space(5.0);
                });

                ScrollArea::vertical()
                .auto_shrink(Vec2b::new(false, false))
                .show(ui, |ui| {

                    for (index, currency) in currencies.iter().enumerate() {
                        match currency {
                            Currency::Native(native) => {
                                if native.symbol.to_lowercase().contains(&self.search_query) {
                                    ui.push_id(index, |ui| {
                                        let cache = SHARED_CACHE.read().unwrap();
                                        let (_, balance) = cache.get_eth_balance(chain_id, owner);
                                        let balance =
                                            format_wei(&balance.to_string(), currency.decimals());
                                        let formated_balance = format!("{:.4}", balance);
                                        let balance_text = RichText::new(format!(
                                            "{} {}",
                                            formated_balance, native.symbol
                                        ))
                                        .size(12.0)
                                        .family(roboto_regular())
                                        .color(Color32::WHITE);

                                        let name = RichText::new(native.name.clone())
                                            .size(15.0)
                                            .family(roboto_regular())
                                            .color(Color32::WHITE);

                                        let button = Button::new(name)
                                        .rounding(10.0)
                                        .sense(Sense::click())
                                        .min_size(vec2(70.0, 25.0));

                                        ui.vertical_centered(|ui| {
                                            if ui.add(button).clicked() {
                                                selected_currency = Some(currency.clone());
                                                self.close();
                                            }
                                            ui.label(balance_text);
                                        });

                                        ui.add_space(5.0);
                                    });
                                }
                            }
                            Currency::ERC20(token) => {
                                if token.symbol.to_lowercase().contains(&self.search_query) {
                                    ui.push_id(index, |ui| {
                                        let cache = SHARED_CACHE.read().unwrap();
                                        let balance = cache.get_erc20_balance(
                                            &chain_id,
                                            &owner,
                                            &token.address,
                                        );
                                        let balance =
                                            format_wei(&balance.to_string(), token.decimals);
                                        let formated_balance = format!("{:.4}", balance);
                                        let balance_text = RichText::new(format!(
                                            "{} {}",
                                            formated_balance, token.symbol
                                        ))
                                        .size(12.0)
                                        .family(roboto_regular())
                                        .color(Color32::WHITE);

                                        let name = RichText::new(token.name.clone())
                                            .size(15.0)
                                            .family(roboto_regular())
                                            .color(Color32::WHITE);

                                        let button = Button::new(name)
                                        .rounding(10.0)
                                        .sense(Sense::click())
                                        .min_size(vec2(70.0, 25.0));

                                        ui.vertical_centered(|ui| {
                                            if ui.add(button).clicked() {
                                                selected_currency = Some(currency.clone());
                                                self.close();
                                            }
                                            ui.label(balance_text);
                                        });
                                        ui.add_space(5.0);
                                    });
                                }

                                // if search string is a valid ethereum address
                                if let Ok(address) = Address::from_str(&self.search_query) {
                                    if ui.button("Add Token").clicked() {
                                        trace!("Adding Token: {:?}", address);
                                        let client = match data.client() {
                                            Some(client) => client,
                                            None => {
                                                let mut state = SHARED_UI_STATE.write().unwrap();
                                                state
                                                    .err_msg
                                                    .show("You are not connected to a node");
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
                            }
                        }
                    }
                });
            });
        selected_currency
    }
}
