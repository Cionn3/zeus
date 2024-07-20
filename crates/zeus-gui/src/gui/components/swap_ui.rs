use eframe::egui::{
    vec2, Align, Button, Color32, FontId, Layout, RichText, TextEdit, Ui,
};
use std::sync::Arc;
use tracing::trace;

use crossbeam::channel::Sender;

use crate::{fonts::roboto_regular, icons::IconTextures};

use super::TokenSelectionWindow;
use zeus_backend::types::Request;
use zeus_chain::{defi_types::currency::Currency, utils::format_wei};
use zeus_shared_types::{
    AppData, cache::SHARED_CACHE, UiState, SHARED_UI_STATE,
};


pub struct SwapUI {
    /// Send Request to the backend
    pub front_sender: Option<Sender<Request>>,

    pub state: UiState,

    /// Currency to swap from
    pub currency_in: Currency,

    /// Currency to swap to
    pub currency_out: Currency,

    pub amount_in: String,

    pub amount_out: String,

    /// Latest Block
    pub block: u64,
}

impl Default for SwapUI {
    fn default() -> Self {
        Self {
            front_sender: None,
            state: UiState::OPEN,
            currency_in: Currency::new_native(1),
            currency_out: Currency::default_erc20(1),
            amount_in: String::new(),
            amount_out: String::new(),
            block: 0,
        }
    }
}

impl SwapUI {

    pub fn amount_in(&mut self) -> &mut String {
        &mut self.amount_in
    }

    pub fn amount_out(&mut self) -> &mut String {
        &mut self.amount_out
    }


    /// Get the input or output selected currency by an id
    pub fn get_currency(&self, id: &str) -> &Currency {
        match id {
            "input" => &self.currency_in,
            "output" => &self.currency_out,
            // * This should not happen
            _ => panic!("Invalid token id, expected 'input' or 'output' but got {}", id),
        }
    }

    /// Replace the input or output currency by an id
    pub fn replace_currency(&mut self, id: &str, currency: Currency) {
        match id {
            "input" => {
                self.currency_in = currency;
            }
            "output" => {
                self.currency_out = currency;
            }
            _ => {}
        }
    }

    /// Give a default input currency based on the selected chain id
    pub fn default_input(&mut self, id: u64) {
        self.currency_in = Currency::new_native(id);
    }

    /// Give a default output currency based on the selected chain id
    pub fn default_output(&mut self, id: u64) {
        self.currency_out = Currency::default_erc20(id);
    }

    /// Show this UI
    ///
    /// This should be called by the [eframe::App::update] method
    pub fn show(
        &mut self,
        ui: &mut Ui,
        data: &mut AppData,
        token_selection: &mut TokenSelectionWindow,
        icons: Arc<IconTextures>,
    ) {
        if self.state.is_close() {
            return;
        }

        let currencies;
        {
            let cache = SHARED_CACHE.read().unwrap();
            currencies = cache
                .currencies
                .get(&data.chain_id.id())
                .unwrap_or(&vec![])
                .clone();
        }

        let swap_text = RichText::new("Swap")
            .family(roboto_regular())
            .size(20.0)
            .color(Color32::WHITE);

        let for_text = RichText::new("For")
            .family(roboto_regular())
            .size(20.0)
            .color(Color32::WHITE);
    

        ui.vertical_centered(|ui| {
            ui.set_max_size(vec2(550.0, 220.0));

            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                let res = ui.add(icons.tx_settings_icon());

                if res.clicked() {
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.tx_settings_on = true;
                }
            });

            ui.label(swap_text);

            ui.horizontal(|ui| {
                ui.add_space(115.0);
                self.amount_field(ui, "input");
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    self.token_button(ui, "input", data, token_selection);
                    self.currency_balance(ui, data, "input");
                });
            });
            ui.add_space(10.0);

            ui.label(for_text);

            ui.horizontal(|ui| {
                ui.add_space(115.0);
                self.amount_field(ui, "output");
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    self.token_button(ui, "output", data, token_selection);
                    self.currency_balance(ui, data, "output");
                });
            });

            let selected = token_selection.show(ui, data, &currencies);
            if let Some(currency) = selected {
                let id = token_selection.get_id();
                self.replace_currency(&id, currency);
    
            }

                self.swap_button(ui, data);

        });
    }

    /// Creates the amount field
    fn amount_field(&mut self, ui: &mut Ui, direction: &str) {
        let font = FontId::new(23.0, roboto_regular());
        let hint = RichText::new("0")
            .color(Color32::WHITE)
            .size(23.0)
            .family(roboto_regular());

        let amount = match direction {
            "input" => self.amount_in(),
            "output" => self.amount_out(),
            _ => panic!("Invalid direction, expected 'input' or 'output' but got {}", direction),
        };

        let field = TextEdit::singleline(amount)
            .font(font)
            .min_size(vec2(100.0, 30.0))
            .text_color(Color32::WHITE)
            .hint_text(hint);

        ui.add(field);
    }

    /// Create the token button
    ///
    /// If clicked it will show the [TokenSelectionWindow]
    fn token_button(
        &mut self,
        ui: &mut Ui,
        currency_id: &str,
        data: &mut AppData,
        token_selection: &mut TokenSelectionWindow
    ) {
        ui.push_id(currency_id, |ui| {

        
        let symbol = self.get_currency(currency_id).symbol();
        let symbol_text = RichText::new(symbol)
            .color(Color32::WHITE)
            .size(15.0)
            .family(roboto_regular());

        let button = Button::new(symbol_text)
            .min_size(vec2(30.0, 15.0))
            .rounding(10.0)
            .stroke((0.3, Color32::WHITE));

        if ui.add(button).clicked() {
            token_selection.set_id(currency_id.to_string());
            token_selection.state.open();
        }

    });
    }

    /// Show the currency balance
    fn currency_balance(
        &mut self,
        ui: &mut Ui,
        data: &mut AppData,
        currency_id: &str,
    ) {
        let balance;
        let currency = self.get_currency(currency_id);
        {
            let chain_id = data.chain_id.id();
            let owner = data.wallet_address();
            let cache = SHARED_CACHE.read().unwrap();
            match currency {
                Currency::Native(_) => {
                    let (_, bal) = cache.get_eth_balance(chain_id, owner);
                    balance = bal;
                }
                Currency::ERC20(token) => {
                   balance = cache.get_erc20_balance(&chain_id, &owner, &token.address);
                }
            }
        }

        let balance_text = RichText::new("Balance:")
        .size(12.0)
        .family(roboto_regular())
        .color(Color32::WHITE);

        let balance = format_wei(&balance.to_string(), currency.decimals());
        let formated_balance = format!("{:.4}", balance);

        ui.horizontal(|ui| {
            ui.label(balance_text);
            ui.add_space(1.0);
            ui.label(formated_balance);
        });

    }

    /// Creates the swap button
    fn swap_button(&mut self, ui: &mut Ui, data: &mut AppData) {
        let text = RichText::new("Swap")
            .size(15.0)
            .family(roboto_regular())
            .color(Color32::WHITE);

        let button = Button::new(text)
            .min_size(vec2(100.0, 30.0))
            .rounding(10.0)
            .stroke((0.3, Color32::WHITE));

        if ui.add(button).clicked() {
            trace!("Swap button clicked, TODO!");
        }

    }
}
