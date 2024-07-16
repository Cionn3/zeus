use eframe::egui::{Color32, ComboBox, RichText, Ui, menu};

use crate::{fonts::roboto_regular, gui::swap_ui::SwapUI, theme::ZeusTheme};
use std::sync::Arc;

use wallet_ui::wallet_ui;
use settings::networks_settings_ui;

use zeus_backend::types::Request;
use zeus_shared_types::{AppData, ErrorMsg, SHARED_UI_STATE, SWAP_UI_STATE};

use crossbeam::channel::Sender;

pub mod misc;
pub mod swap_ui;
pub mod wallet_ui;
pub mod settings;

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

    /// Show the Side Panel Menu
    /// 
    /// This should be called by the [eframe::App::update] method
    pub fn side_panel_menu(&mut self, ui: &mut Ui, data: &mut AppData) {
        let swap = RichText::new("Swap").family(roboto_regular()).size(20.0);

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
                self.swap_ui.open = true;
            }           
        });
    }



    /// Show the wallet selection UI
    /// 
    /// This should be called by the [eframe::App::update] method
    pub fn wallet_ui(&mut self, ui: &mut Ui, data: &mut AppData) {
        wallet_ui(ui, data, &self);
    }

    /// Show Network Settings UI
    /// 
    /// This should be called by the [eframe::App::update] method
    /// 
    /// Depending on the state of the [SHARED_UI_STATE], this will show the network settings UI
    pub fn show_network_settings_ui(&mut self, ui: &mut Ui, data: &mut AppData) {
        networks_settings_ui(ui, data, self.theme.icons.clone());
    }

    /// Chain Selection
    /// 
    /// This should be called by the [eframe::App::update] method
    pub fn select_chain(&mut self, ui: &mut Ui, data: &mut AppData) {
        let chain_ids = data.chain_ids.clone();
        ui.horizontal(|ui| {
            ui.add(self.theme.icons.chain_icon(&data.chain_id.id()));

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

    /// Show the Settings Menu
    /// 
    /// This should be called by the [eframe::App::update] method
    pub fn settings_menu(&mut self, ui: &mut Ui) {

        let settings = RichText::new("Settings")
        .family(roboto_regular())
        .size(14.0)
        .color(Color32::WHITE);

        let wallet_settings = RichText::new("Wallet Settings")
        .family(roboto_regular())
        .size(14.0)
        .color(Color32::WHITE);

        let network_settings = RichText::new("Network Settings")
        .family(roboto_regular())
        .size(14.0)
        .color(Color32::WHITE);

        menu::bar(ui, |ui| {
            ui.menu_button(settings, |ui| {

                // Wallet Settings sub-menu
                ui.menu_button(wallet_settings, |ui| {
                    if ui.button("New Wallet").clicked() {
                        ui.close_menu();
                        let mut state = SHARED_UI_STATE.write().unwrap();
                        state.new_wallet_window_on = true;
                    }

                    if ui.button("Import Wallet").clicked() {
                        ui.close_menu();
                        let mut state = SHARED_UI_STATE.write().unwrap();
                        state.import_wallet_window_on = true;
                    }

                    if ui.button("View Key").clicked() {
                        ui.close_menu();
                        let mut state = SHARED_UI_STATE.write().unwrap();
                        state.export_key_ui = true;
                    }
                });

                // Network Settings
                if ui.button(network_settings).clicked() {
                    ui.close_menu();
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.network_settings = true;
                }
            });
        });
    }
}
