use eframe::egui::{vec2, Button, Color32, ComboBox, FontId, RichText, Sense, TextEdit, Ui};

use crate::{fonts::roboto_regular, gui::swap_ui::SwapUI, theme::ZeusTheme};
use std::sync::Arc;

use wallet_ui::wallet_ui;

use zeus_backend::types::Request;
use zeus_shared_types::{AppData, ErrorMsg, SHARED_UI_STATE, SWAP_UI_STATE};

use crossbeam::channel::Sender;
use tracing::trace;

pub mod misc;
pub mod swap_ui;
pub mod wallet_ui;

/// The Graphical User Interface for [crate::ZeusApp]
pub struct GUI {
    /// Send data to backend
    pub sender: Option<Sender<Request>>,

    pub swap_ui: SwapUI,

    pub theme: Arc<ZeusTheme>,
}

impl GUI {
    pub fn default() -> Self {
        Self {
            sender: None,
            swap_ui: SwapUI::default(),
            theme: Arc::new(ZeusTheme::default()),
        }
    }

    /// Send a request to the backend
    pub fn send_request(&self, request: Request) {
        if let Some(sender) = &self.sender {
            match sender.send(request) {
                Ok(_) => {}
                Err(e) => {
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.err_msg = ErrorMsg::new(true, e);
                }
            }
        }
    }

    /// Render and handle the menu
    pub fn menu(&mut self, ui: &mut Ui, data: &mut AppData) {
        let swap = RichText::new("Swap").family(roboto_regular()).size(20.0);
        let settings = RichText::new("Settings")
            .family(roboto_regular())
            .size(20.0);
        let networks = RichText::new("Networks")
            .family(roboto_regular())
            .size(15.0);
        let base_fee = RichText::new("Base Fee")
            .family(roboto_regular())
            .size(15.0);

        ui.vertical(|ui| {
            ui.label(base_fee);
            ui.label(
                RichText::new(&data.next_block.format_gwei())
                    .family(roboto_regular())
                    .size(15.0),
            );
            ui.add_space(10.0);

            if ui.label(swap).clicked() {
                let mut state = SHARED_UI_STATE.write().unwrap();
                state.networks_on = false;
                state.swap_ui_on = true;
            }

            ui.add_space(10.0);

            ui.collapsing(settings, |ui| {
                if ui.label(networks).clicked() {
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.swap_ui_on = false;
                    state.networks_on = true;
                }
            });
        });
    }

    /// Render Network Settings UI
    pub fn networks_ui(&mut self, ui: &mut Ui, data: &mut AppData) {
        {
            let state = SHARED_UI_STATE.read().unwrap();
            if !state.networks_on {
                return;
            }
        }

        let description = RichText::new("RPC Endpoints, Currently only supports Websockets")
            .family(roboto_regular())
            .color(Color32::WHITE)
            .size(20.0);
        let font = FontId::new(15.0, roboto_regular());


        ui.vertical_centered(|ui| {
            ui.set_max_size(vec2(400.0, 500.0));

            ui.add_space(60.0);
            ui.label(description);
            ui.add_space(20.0);

                for network in data.rpc.iter_mut() {
                    let label = RichText::new(network.chain_name())
                        .family(roboto_regular())
                        .size(15.0)
                        .color(Color32::WHITE);
                    ui.label(label);

                    ui.add_space(10.0);

                    let text_field = TextEdit::singleline(&mut network.url)
                        .font(font.clone())
                        .text_color(Color32::WHITE);
                    ui.add(text_field);
                    ui.add_space(10.0);
                }
          
                let save = RichText::new("Save")
                    .family(roboto_regular())
                    .size(15.0)
                    .color(Color32::WHITE);

                let button = Button::new(save)
                    .rounding(10.0)
                    .sense(Sense::click())
                    .min_size(vec2(70.0, 25.0));

                if ui.add(button).clicked() {
                    match data.save_rpc() {
                        Ok(_) => {
                            trace!("Saved RPC Endpoints");
                        }
                        Err(e) => {
                            let mut state = SHARED_UI_STATE.write().unwrap();
                            state.err_msg = ErrorMsg::new(true, e);
                        }
                    }
                }
           
        });
    }

    /// Render the UI repsonsible for managing the wallets
    pub fn render_wallet_ui(&mut self, ui: &mut Ui, data: &mut AppData) {
        wallet_ui(ui, data, &self);
    }

    /// Chain Selection
    pub fn select_chain(&mut self, ui: &mut Ui, data: &mut AppData) {
        let chain_ids = data.chain_ids.clone();
        ui.horizontal(|ui| {
            ui.add(self.theme.icons.chain_icon(data.chain_id.id()));

            ComboBox::from_label("")
                .selected_text(data.chain_id.name())
                .show_ui(ui, |ui| {
                    for chain_id in chain_ids {
                        if ui
                            .selectable_value(&mut data.chain_id, chain_id.clone(), chain_id.name())
                            .clicked()
                        {
                            // Send a request to the backend to get the client
                            self.send_request(Request::GetClient {
                                chain_id: chain_id.clone(),
                                rpcs: data.rpc.clone(),
                                clients: data.ws_client.clone(),
                            });

                            let mut swap_ui_state = SWAP_UI_STATE.write().unwrap();
                            swap_ui_state.default_input(chain_id.id());
                            swap_ui_state.default_output(chain_id.id());
                        }
                    }
                });
            ui.add(
                self.theme
                    .icons
                    .connected_icon(data.connected(data.chain_id.id())),
            );
        });
    }
}
