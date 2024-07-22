use crate::{fonts::roboto_regular, theme::THEME};
use eframe::egui::{vec2, Align2, Button, Color32, RichText, Sense, TextEdit, Ui, Window};

use super::TokenSelectionWindow;
use crossbeam::channel::Sender;
use zeus_backend::types::Request;
use zeus_chain::{alloy::primitives::{Address, U256}, defi_types::currency::Currency, format_wei};
use zeus_shared_types::{cache::SHARED_CACHE, AppData, UiState};

/// The Send Crypto Screen UI

pub struct SendCryptoScreen {
    pub state: UiState,
    pub selected_currency: Currency,
    token_selection_window: TokenSelectionWindow,
    amount: String,
    recipient: String,
}

impl SendCryptoScreen {
    pub fn new(sender: Sender<Request>) -> Self {
        Self {
            state: UiState::default(),
            selected_currency: Currency::default(),
            token_selection_window: TokenSelectionWindow::new(sender),
            amount: String::new(),
            recipient: String::new(),
        }
    }

    /// Give a default input currency based on the selected chain id
    pub fn default_input(&mut self, id: u64) {
        self.selected_currency = Currency::new_native(id);
    }

    /// Get balance of the selected currency
    fn get_balance(&self, chain_id: u64, owner: Address) -> U256 {
        match &self.selected_currency {
            Currency::Native(_) => {
                let cache = SHARED_CACHE.read().unwrap();
                let (_, balance) = cache.get_eth_balance(chain_id, owner);
                balance
            }
            Currency::ERC20(token) => {
                let cache = SHARED_CACHE.read().unwrap();
                let balance = cache.get_erc20_balance(&chain_id, &owner, &token.address);
                balance
            }
        }
    }

    /// Show this UI
    ///
    /// This should be called by the [eframe::App::update] method
    pub fn show(&mut self, ui: &mut Ui, data: &mut AppData) {
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

        let send_crypto = RichText::new("Send Crypto")
            .family(roboto_regular())
            .size(20.0);

        let send = RichText::new("Send").family(roboto_regular()).size(20.0);
        let cancel = RichText::new("Cancel").family(roboto_regular()).size(20.0);
        let token = RichText::new("Token").family(roboto_regular()).size(15.0);
        let amount = RichText::new("Amount").family(roboto_regular()).size(15.0);
        let recipient = RichText::new("Recipient").family(roboto_regular()).size(15.0);

        let send_button = Button::new(send)
            .rounding(10.0)
            .sense(Sense::click())
            .min_size(vec2(70.0, 25.0));

        let cancel_button = Button::new(cancel)
            .rounding(10.0)
            .sense(Sense::click())
            .min_size(vec2(70.0, 25.0));

        let chain_id = data.chain_id.id();
        let owner = data.wallet_address();

        let balance = self.get_balance(chain_id, owner);
        let balance = format_wei(&balance.to_string(), self.selected_currency.decimals().clone());
        let balance = format!("{:.4}", balance);

        let amount_edit = TextEdit::singleline(&mut self.amount)
        .hint_text(&format!("{} {} Available", balance, &self.selected_currency.symbol()))
        .min_size(vec2(150.0, 25.0))
        .desired_width(150.0);

        let recipient_edit = TextEdit::singleline(&mut self.recipient)
            .min_size(vec2(150.0, 25.0))
            .desired_width(150.0);

        Window::new(send_crypto)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .resizable(false)
            .collapsible(false)
            .fade_in(true)
            .fade_out(true)
            .show(ui.ctx(), |ui| {
                ui.set_min_size(vec2(300.0, 200.0));

                ui.vertical_centered(|ui| {
                    let name = RichText::new(self.selected_currency.name().clone())
                        .family(roboto_regular())
                        .size(14.0)
                        .color(Color32::WHITE);

                    let icon = THEME.icons.currency_icon(chain_id);

                    let currency_button = Button::image_and_text(icon, name)
                        .rounding(10.0)
                        .sense(Sense::click())
                        .min_size(vec2(75.0, 20.0));

                        ui.label(token);
                        ui.add_space(2.0);
                        if ui.add(currency_button).clicked() {
                            self.token_selection_window.state.open();
                        }

                        ui.add_space(15.0);

                        ui.label(amount);
                        ui.add_space(2.0);
                        ui.add(amount_edit);

                        ui.add_space(15.0);
                        ui.label(recipient);
                        ui.add_space(2.0);
                        ui.add(recipient_edit);
                        ui.add_space(15.0);
                        // TODO: Add Saved Contacts

                    let selected = self.token_selection_window.show(ui, data, &currencies);
                    if let Some(selected) = selected {
                        self.selected_currency = selected;
                    }

                    if ui.add(send_button).clicked() {
                        // TODO
                    }
                    ui.add_space(15.0);

                    if ui.add(cancel_button).clicked() {
                        self.state.close();
                    }
                });
            });
    }
}