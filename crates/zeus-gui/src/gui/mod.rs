use eframe::egui::{menu, Button, Color32, ComboBox, RichText, Ui, Sense, vec2};

use crate::{fonts::roboto_regular, theme::ZeusTheme};
use std::sync::Arc;

use components::{*, send_crypto_screen::SendCryptoScreen, swap_ui::SwapUI, wallet::*};

use zeus_backend::types::Request;
use zeus_shared_types::{AppData, SHARED_UI_STATE, SWAP_UI_STATE};

use crossbeam::channel::Sender;

pub mod misc;
pub mod components;

/// The Graphical User Interface for [crate::ZeusApp]
pub struct GUI {
    /// Send data to backend
    pub sender: Sender<Request>,

    pub token_selection_window: TokenSelectionWindow,

    pub network_settings: NetworkSettings,

    pub swap_ui: SwapUI,

    pub send_screen: SendCryptoScreen,

    pub wallet_ui: WalletUI,

    pub theme: Arc<ZeusTheme>,
}

impl GUI {
    pub fn new(sender: Sender<Request>) -> Self {
        Self {
            sender: sender.clone(),
            token_selection_window: TokenSelectionWindow::new(sender.clone()),
            network_settings: NetworkSettings::new(),
            swap_ui: SwapUI::new(sender.clone()),
            send_screen: SendCryptoScreen::new(sender.clone()),
            wallet_ui: WalletUI::new(sender.clone()),
            theme: Arc::new(ZeusTheme::default()),
        }
    }

    /// Send a request to the backend
    pub fn send_request(&self, request: Request) {
            match self.sender.send(request) {
                Ok(_) => {}
                Err(e) => {
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.err_msg.show(e);
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
                self.swap_ui.state.open();
            }           
        });
    }



    /// Show the wallet UI
    /// 
    /// This should be called by the [eframe::App::update] method
    pub fn wallet_ui(&mut self, ui: &mut Ui, data: &mut AppData) {

        if data.logged_in {
            self.wallet_ui.state.open();
        }

        // show the available wallets
        self.wallet_ui.show(ui, data, self.theme.icons.clone());

        // show the create new wallet ui
        self.wallet_ui.create_wallet_ui.show(ui, data);

        // show the import wallet ui
        self.wallet_ui.import_wallet_ui.show(ui, data);

        // show the view key ui
        self.wallet_ui.view_key_ui.show(ui, data);


    }

    /// Show Network Settings UI
    /// 
    /// This should be called by the [eframe::App::update] method
    pub fn show_network_settings_ui(&mut self, ui: &mut Ui, data: &mut AppData) {
        self.network_settings.show(ui, data, self.theme.icons.clone());
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
                            let req = Request::client(chain_id.clone(), data.rpc.clone());
                            self.send_request(req);

                            let mut swap_ui_state = SWAP_UI_STATE.write().unwrap();
                            swap_ui_state.default_input(chain_id.id());
                            swap_ui_state.default_output(chain_id.id());
                        }
                    }
                });
            ui.add(
                self.theme
                    .icons
                    .connected_icon(data.connected()),
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
                        self.wallet_ui.create_wallet_ui.state.open();
                    }

                    if ui.button("Import Wallet").clicked() {
                        ui.close_menu();
                        self.wallet_ui.import_wallet_ui.state.open();
                    }

                    if ui.button("View Key").clicked() {
                        ui.close_menu();
                        self.wallet_ui.view_key_ui.state.open();
                    }
                    // TODO: Rename and Hide Wallet
                });

                // Network Settings
                if ui.button(network_settings).clicked() {
                    ui.close_menu();
                    self.network_settings.state.open();
                }
            });
        });
    }

    /// Send Button
    /// 
    /// If clicked user is prompted to the [SendCryptoScreen]
    pub fn send_crypto_button(&mut self, ui: &mut Ui, data: &mut AppData) {
        let send = RichText::new("Send")
        .family(roboto_regular())
        .size(18.0)
        .color(Color32::WHITE);

        let send_icon = self.theme.icons.clone().send_icon();

        let send_button = Button::image_and_text(send_icon, send)
        .rounding(10.0)
        .sense(Sense::click())
        .min_size(vec2(75.0, 25.0));

        if ui.add(send_button).clicked() {
            self.send_screen.state.open();
        }

        self.send_screen.show(ui, data);

    }
}
