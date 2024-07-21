use crate::{fonts::roboto_regular, theme::THEME};
use eframe::egui::{vec2, Align2, Button, Color32, RichText, Sense, Ui, Window};

use zeus_shared_types::{
    cache::SHARED_CACHE, AppData, UiState};
use zeus_chain::defi_types::currency::Currency;
use super::TokenSelectionWindow;
use crossbeam::channel::Sender;
use zeus_backend::types::Request;



/// The Send Crypto Screen UI

pub struct SendCryptoScreen {
    pub state: UiState,
    pub selected_currency: Currency,
    token_selection_window: TokenSelectionWindow,
}

impl SendCryptoScreen {
    pub fn new(sender: Sender<Request>) -> Self {
        Self {
            state: UiState::default(),
            selected_currency: Currency::default(),
            token_selection_window: TokenSelectionWindow::new(sender),
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

        let send_button = Button::new(send)
            .rounding(10.0)
            .sense(Sense::click())
            .min_size(vec2(70.0, 25.0));

        let cancel_button = Button::new(cancel)
            .rounding(10.0)
            .sense(Sense::click())
            .min_size(vec2(70.0, 25.0));

        Window::new(send_crypto)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .resizable(false)
            .collapsible(false)
            .fade_in(true)
            .fade_out(true)
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    // TODO: Update the currency when we change chain id
                    let name = RichText::new(self.selected_currency.name().clone())
                        .family(roboto_regular())
                        .size(14.0)
                        .color(Color32::WHITE);

                    let currency_button = Button::new(name)
                        .rounding(10.0)
                        .sense(Sense::click())
                        .min_size(vec2(75.0, 30.0));

                    if ui.add(currency_button).clicked() {
                        self.token_selection_window.state.open();
                    }

                    let selected = self.token_selection_window.show(ui, data, &currencies);
                    if let Some(selected) = selected {
                        self.selected_currency = selected;
                    }

                    ui.set_min_size(vec2(300.0, 200.0));
                    if ui.add(send_button).clicked() {
                        // TODO
                    }

                    if ui.add(cancel_button).clicked() {
                        self.state.close();
                    }
                });
            });
    }

    
}
