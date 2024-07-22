pub mod send_crypto_screen;
pub mod swap_ui;
pub mod wallet;

use crate::{fonts::roboto_regular, icons::IconTextures, theme::THEME};
use crossbeam::channel::Sender;
use eframe::egui::{
    emath::Vec2b, vec2, Align, Align2, Button, Color32, FontId, Layout, RichText, ScrollArea, Sense, TextEdit, Ui, Window
};
use std::{str::FromStr, sync::Arc};
use tracing::trace;
use zeus_backend::types::*;
use zeus_chain::{alloy::primitives::Address, defi_types::currency::Currency, utils::format_wei};
use zeus_shared_types::{cache::SHARED_CACHE, AppData, UiState, SHARED_UI_STATE};

pub struct TokenSelectionWindow {
    pub state: UiState,

    pub search_query: String,

    pub sender: Sender<Request>,

    pub currency_id: String,
}

impl TokenSelectionWindow {
    pub fn new(sender: Sender<Request>) -> Self {
        Self {
            state: UiState::default(),
            search_query: String::new(),
            sender,
            currency_id: String::new(),
        }
    }

    pub fn set_id(&mut self, currency_id: String) {
        self.currency_id = currency_id
    }

    pub fn get_id(&self) -> String {
        self.currency_id.clone()
    }

    /// Send a request to the backend
    pub fn send_request(&self, request: Request) {
            match self.sender.send(request) {
                Ok(_) => {}
                Err(e) => {
                    trace!("Error sending request: {}", e);
                }
            }
        
    }

    /// Show This [TokenSelectionWindow] UI
    ///
    /// # Arguments
    ///
    /// * `ui` - The egui Ui
    /// * `data` - The AppData
    /// * `currencies` - The list of currencies
    ///
    /// # Returns
    ///
    /// The selected currency
    fn show(
        &mut self,
        ui: &mut Ui,
        data: &AppData,
        currencies: &Vec<Currency>,
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
                                            let (_, balance) =
                                                cache.get_eth_balance(chain_id, owner);
                                            let balance = format_wei(
                                                &balance.to_string(),
                                                currency.decimals(),
                                            );
                                            let formated_balance = format!("{:.4}", balance);
                                            let balance_text = RichText::new(format!(
                                                "{} {}",
                                                formated_balance, native.symbol
                                            ))
                                            .size(15.0)
                                            .family(roboto_regular())
                                            .color(Color32::WHITE);

                                            let name = RichText::new(native.name.clone())
                                                .size(15.0)
                                                .family(roboto_regular())
                                                .color(Color32::WHITE);

   
                                            let icon = THEME.icons.currency_icon(chain_id);

                                            let button = Button::image_and_text(icon, name)
                                                .rounding(10.0)
                                                .sense(Sense::click())
                                                .min_size(vec2(70.0, 25.0));

                                           
                                                ui.horizontal(|ui| {
                                        
                                                if ui.add(button).clicked() {
                                                    selected_currency = Some(currency.clone());
                                                    self.state.close();
                                                }
                                                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                                ui.label(balance_text);
                                            });
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
                                            // TODO: use something like numformat
                                            // to deal with very large numbers
                                            let balance =
                                                format_wei(&balance.to_string(), token.decimals);
                                            let formated_balance = format!("{:.4}", balance);
                                            let balance_text = RichText::new(format!(
                                                "{} {}",
                                                formated_balance, token.symbol
                                            ))
                                            .size(15.0)
                                            .family(roboto_regular())
                                            .color(Color32::WHITE);

                                            let name = RichText::new(token.name.clone())
                                                .size(15.0)
                                                .family(roboto_regular())
                                                .color(Color32::WHITE);

                                            // Use the currency icon cause the
                                            // erc20 placeholder is diplayed blurry 
                                            let icon = THEME.icons.currency_icon(chain_id);

                                            let button = Button::image_and_text(icon, name)
                                                .rounding(10.0)
                                                .sense(Sense::click())
                                                .min_size(vec2(70.0, 25.0));

                                            
                                                ui.horizontal(|ui| {
                                                if ui.add(button).clicked() {
                                                    selected_currency = Some(currency.clone());
                                                    self.state.close();
                                                }
                                                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                                ui.label(balance_text);
                                            });
                                        });
                                           
                                            ui.add_space(5.0);
                                        
                                    });
                                    }
                                }
                            }
                        }

                        let add_token_text = RichText::new("Add Token")
                            .size(15.0)
                            .family(roboto_regular())
                            .color(Color32::WHITE);

                        let add_token_button = Button::new(add_token_text)
                            .rounding(10.0)
                            .sense(Sense::click())
                            .min_size(vec2(70.0, 25.0));

                        // if search string is a valid ethereum address
                        if let Ok(address) = Address::from_str(&self.search_query) {
                            ui.vertical_centered(|ui| {

                            if ui.add(add_token_button).clicked() {
                                trace!("Adding Token: {:?}", address);
                                let client = match data.client().clone() {
                                    Some(client) => client,
                                    None => {
                                        let mut state = SHARED_UI_STATE.write().unwrap();
                                        state.err_msg.show("You are not connected to a node");
                                        return;
                                    }
                                };
                                let owner = data.wallet_address();
                                let chain_id = data.chain_id.id();

                                let req = Request::erc20_token(self.get_id(), owner, address, chain_id, client );
                                self.send_request(req);

                                self.state.close();
                            }
                        });
                        }
                    });
            });
        selected_currency
    }
}



pub struct NetworkSettings {
    pub state: UiState,
}

impl NetworkSettings {
    pub fn new() -> Self {
        Self {
            state: UiState::default(),
        }
    }

    /// Show This [NetworkSettings] UI
    ///
    /// # Arguments
    ///
    /// * `ui` - The egui Ui
    /// * `data` - The AppData
    /// * `icons` - The IconTextures
    pub fn show(&mut self, ui: &mut Ui, data: &mut AppData, icons: Arc<IconTextures>) {
        if self.state.is_close() {
            return;
        }

        let settings = RichText::new("Network Settings")
            .family(roboto_regular())
            .size(20.0)
            .color(Color32::WHITE);

        let save = RichText::new("Save")
            .family(roboto_regular())
            .size(15.0)
            .color(Color32::WHITE);

        let save_button = Button::new(save)
            .rounding(10.0)
            .sense(Sense::click())
            .min_size(vec2(70.0, 25.0));

        let font = FontId::new(15.0, roboto_regular());


        Window::new(settings)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.set_min_size(vec2(250.0, 320.0));

                    ui.add_space(20.0);

                    for network in data.rpc.iter_mut() {
                        ui.horizontal(|ui| {
                            ui.add_space(60.0);
                            ui.add(icons.chain_icon(&network.chain_id));
                            ui.add_space(3.0);
                            let text = RichText::new(network.chain_name())
                                .family(roboto_regular())
                                .size(15.0)
                                .color(Color32::WHITE);
                            ui.label(text);
                        });

                        ui.add_space(10.0);
                        let text_edit = TextEdit::singleline(&mut network.url)
                            .font(font.clone())
                            .text_color(Color32::WHITE)
                            .desired_width(200.0);
                        ui.add(text_edit);
                        ui.add_space(10.0);
                    }

                    if ui.add(save_button).clicked() {
                        match data.save_rpc() {
                            Ok(_) => {
                                trace!("Network settings saved");
                                self.state.close();
                            }
                            Err(e) => {
                                let mut state = SHARED_UI_STATE.write().unwrap();
                                state.err_msg.show(&format!("Error saving network settings: {}", e));
                                self.state.close();
                            }
                        }
                    }
                });
            });                      
}
}