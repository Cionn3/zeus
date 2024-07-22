use eframe::{
    egui::{Align2, Button, Color32, ComboBox, FontId, RichText, Sense, TextEdit, Ui, Window},
    epaint::vec2,
};
use std::{collections::HashMap, sync::Arc};

use crate::{fonts::roboto_regular, icons::IconTextures};
use crossbeam::channel::Sender;
use tracing::trace;
use zeus_backend::types::Request;
use zeus_chain::alloy::primitives::utils::format_ether;
use zeus_core::Credentials;
use zeus_shared_types::{AppData, UiState, SHARED_UI_STATE};

/// UI for viewing a private key
pub struct ViewPrivateKeyUI {
    pub state: UiState,
    pub show_key: UiState,
    pub exported_key: String,
    pub credentials: Credentials,
}

impl ViewPrivateKeyUI {
    pub fn new() -> Self {
        Self {
            state: UiState::default(),
            show_key: UiState::default(),
            exported_key: String::new(),
            credentials: Credentials::default(),
        }
    }

    /// Show This UI
    ///
    /// This should be called by the [eframe::App::update] method
    pub fn show(&mut self, ui: &mut Ui, data: &mut AppData) {
        if self.state.is_close() {
            return;
        }

        if data.profile.current_wallet.is_none() {
            self.state.close();
            let mut state = SHARED_UI_STATE.write().unwrap();
            state.err_msg.show("No wallet selected");
            return;
        }

        let window_title = RichText::new("View Key")
            .family(roboto_regular())
            .size(20.0)
            .color(Color32::WHITE);

        let heading = RichText::new("Confirm Your Credentials")
            .family(roboto_regular())
            .size(20.0)
            .color(Color32::WHITE);

        let username = RichText::new("Username:")
            .family(roboto_regular())
            .size(18.0)
            .color(Color32::WHITE);

        let password = RichText::new("Password:")
            .family(roboto_regular())
            .size(18.0)
            .color(Color32::WHITE);

        let font = FontId::new(15.0, roboto_regular());

        Window::new(window_title)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .collapsible(false)
            .fade_in(true)
            .fade_out(true)
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(heading);
                    ui.add_space(10.0);

                    {
                        let username_field = TextEdit::singleline(self.credentials.user_mut())
                            .desired_width(150.0)
                            .min_size(vec2(150.0, 25.0))
                            .font(font.clone());
                        ui.label(username);
                        ui.add_space(5.0);
                        ui.add(username_field);
                        ui.add_space(10.0);
                    }

                    {
                        let password_field = TextEdit::singleline(self.credentials.passwd_mut())
                            .desired_width(150.0)
                            .min_size(vec2(150.0, 25.0))
                            .password(true)
                            .font(font.clone());
                        ui.label(password);
                        ui.add_space(5.0);
                        ui.add(password_field);
                        ui.add_space(10.0);
                    }

                    let view_key_text = RichText::new("View Key")
                        .family(roboto_regular())
                        .size(15.0)
                        .color(Color32::WHITE);

                    let view_button = Button::new(view_key_text)
                        .rounding(10.0)
                        .sense(Sense::click())
                        .min_size(vec2(70.0, 30.0));

                    if ui.add(view_button).clicked() {
                        let wallet = data.profile.current_wallet.clone().unwrap();
                        self.credentials.copy_passwd_to_confirm();

                        let key = match data.profile.export_wallet(wallet, self.credentials.clone())
                        {
                            Ok(key) => key,
                            Err(e) => {
                                let mut state = SHARED_UI_STATE.write().unwrap();
                                state.err_msg.show(e);
                                return;
                            }
                        };

                        self.credentials.clear();
                        self.exported_key = key;

                        self.show_key.open();
                    }
                    ui.add_space(10.0);
                    self.show_key(ui);

                    let close_text = RichText::new("Close")
                        .family(roboto_regular())
                        .size(15.0)
                        .color(Color32::WHITE);

                    let close_button = Button::new(close_text)
                        .rounding(10.0)
                        .sense(Sense::click())
                        .min_size(vec2(70.0, 30.0));

                    if ui.add(close_button).clicked() {
                        self.state.close();
                    }
                });
            });
    }

    fn show_key(&mut self, ui: &mut Ui) {
        if self.show_key.is_close() {
            return;
        }

        let window_title = RichText::new("Exported Key")
            .family(roboto_regular())
            .size(20.0)
            .color(Color32::WHITE);

        Window::new(window_title)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .collapsible(false)
            .fade_in(true)
            .fade_out(true)
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    let key_text = RichText::new(&self.exported_key)
                        .family(roboto_regular())
                        .size(15.0)
                        .color(Color32::WHITE);

                    ui.label(key_text);
                    ui.add_space(10.0);

                    let close_text = RichText::new("Close")
                        .family(roboto_regular())
                        .size(15.0)
                        .color(Color32::WHITE);

                    let close_button = Button::new(close_text)
                        .rounding(10.0)
                        .sense(Sense::click())
                        .min_size(vec2(70.0, 30.0));

                    if ui.add(close_button).clicked() {
                        self.exported_key.clear();
                        self.show_key.close();
                    }
                });
            });
    }
}

/// UI for importing a wallet from a private key
pub struct ImportWalletUI {
    pub state: UiState,
    pub wallet_name: String,
    pub private_key: String,
    pub sender: Sender<Request>,
}

impl ImportWalletUI {
    pub fn new(sender: Sender<Request>) -> Self {
        Self {
            state: UiState::default(),
            wallet_name: String::new(),
            private_key: String::new(),
            sender,
        }
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

    /// Show this UI
    ///
    /// This should be called by the [eframe::App::update] method
    pub fn show(&mut self, ui: &mut Ui, data: &mut AppData) {
        if self.state.is_close() {
            return;
        }

        let font = FontId::new(15.0, roboto_regular());

        Window::new("Import Wallet")
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .collapsible(false)
            .fade_in(true)
            .fade_out(true)
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);

                    let private_key = RichText::new("Private Key:")
                        .family(roboto_regular())
                        .size(18.0)
                        .color(Color32::WHITE);

                    let private_key_field = TextEdit::singleline(&mut self.private_key)
                        .desired_width(150.0)
                        .min_size(vec2(150.0, 25.0))
                        .password(true)
                        .font(font.clone());

                    let name_text = RichText::new("Wallet Name (Optional):")
                        .family(roboto_regular())
                        .size(18.0)
                        .color(Color32::WHITE);

                    let name_field = TextEdit::singleline(&mut self.wallet_name)
                        .desired_width(150.0)
                        .min_size(vec2(150.0, 25.0))
                        .font(font);

                    ui.label(name_text);
                    ui.add_space(5.0);
                    ui.add(name_field);
                    ui.add_space(15.0);
                    ui.label(private_key);
                    ui.add_space(5.0);
                    ui.add(private_key_field);
                    ui.add_space(15.0);

                    let import_text = RichText::new("Import Wallet")
                        .family(roboto_regular())
                        .size(15.0)
                        .color(Color32::WHITE);

                    let import_button = Button::new(import_text)
                        .rounding(10.0)
                        .sense(Sense::click())
                        .min_size(vec2(70.0, 30.0));

                    let close_text = RichText::new("Close")
                        .family(roboto_regular())
                        .size(15.0)
                        .color(Color32::WHITE);

                    let close_button = Button::new(close_text)
                        .rounding(10.0)
                        .sense(Sense::click())
                        .min_size(vec2(70.0, 30.0));

                    if ui.add(import_button).clicked() {
                        match data.profile.import_wallet(
                            self.wallet_name.clone(),
                            HashMap::new(),
                            self.private_key.clone(),
                        ) {
                            Ok(_) => {
                                self.state.close();
                                self.wallet_name.clear();
                                self.private_key.clear();
                            }
                            Err(e) => {
                                let mut state = SHARED_UI_STATE.write().unwrap();
                                state.err_msg.show(e);
                            }
                        }

                        self.send_request(Request::SaveProfile(data.profile.clone()));
                    }
                    ui.add_space(15.0);

                    if ui.add(close_button).clicked() {
                        self.state.close();
                        self.private_key.clear();
                    }
                });
            });
    }
}

/// UI For creating a new wallet
pub struct CreateNewWalletUI {
    pub state: UiState,
    pub wallet_name: String,
    pub sender: Sender<Request>,
}

impl CreateNewWalletUI {
    pub fn new(sender: Sender<Request>) -> Self {
        Self {
            state: UiState::default(),
            wallet_name: String::new(),
            sender
        }
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

    /// Show this UI
    ///
    /// This should be called by the [eframe::App::update] method
    pub fn show(&mut self, ui: &mut Ui, data: &mut AppData) {
        if self.state.is_close() {
            return;
        }

        let font = FontId::new(15.0, roboto_regular());

        Window::new("New Wallet")
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .collapsible(false)
            .fade_in(true)
            .fade_out(true)
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);

                    let wallet_name = RichText::new("Wallet Name (Optional):")
                        .family(roboto_regular())
                        .size(18.0)
                        .color(Color32::WHITE);

                    let name_field = TextEdit::singleline(&mut self.wallet_name)
                        .desired_width(150.0)
                        .min_size(vec2(150.0, 25.0))
                        .font(font);

                    ui.label(wallet_name);
                    ui.add_space(5.0);
                    ui.add(name_field);
                    ui.add_space(25.0);

                    let create_text = RichText::new("Create Wallet")
                        .family(roboto_regular())
                        .size(15.0)
                        .color(Color32::WHITE);

                    let create_button = Button::new(create_text)
                        .rounding(10.0)
                        .sense(Sense::click())
                        .min_size(vec2(70.0, 30.0));

                    let close_text = RichText::new("Close")
                        .family(roboto_regular())
                        .size(15.0)
                        .color(Color32::WHITE);

                    let close_button = Button::new(close_text)
                        .rounding(10.0)
                        .sense(Sense::click())
                        .min_size(vec2(70.0, 30.0));

                    if ui.add(create_button).clicked() {
                        match data.profile.new_wallet(self.wallet_name.clone()) {
                            Ok(_) => {
                                self.state.close();
                                self.wallet_name.clear();
                            }
                            Err(e) => {
                                let mut state = SHARED_UI_STATE.write().unwrap();
                                state.err_msg.show(e);
                            }
                        }

                        self.send_request(Request::SaveProfile(data.profile.clone()));
                    }
                    ui.add_space(15.0);

                    if ui.add(close_button).clicked() {
                        self.state.close();
                        self.wallet_name.clear();
                    }
                });
            });
    }
}

/// UI to prompt the user to create a new random wallet or import one
#[derive(Clone, Default)]
pub struct NewWalletUI {
    pub state: UiState,
}

impl NewWalletUI {
    pub fn new() -> Self {
        Self {
            state: UiState::default(),
        }
    }
}

pub struct WalletUI {
    pub state: UiState,
    pub new_wallet_ui: UiState,
    pub view_key_ui: ViewPrivateKeyUI,
    pub import_wallet_ui: ImportWalletUI,
    pub create_wallet_ui: CreateNewWalletUI,
}

impl WalletUI {
    pub fn new(sender: Sender<Request>) -> Self {
        Self {
            state: UiState::default(),
            new_wallet_ui: UiState::default(),
            view_key_ui: ViewPrivateKeyUI::new(),
            import_wallet_ui: ImportWalletUI::new(sender.clone()),
            create_wallet_ui: CreateNewWalletUI::new(sender.clone()),
        }
    }

    /// Show this UI
    ///
    /// This should be called by the [eframe::App::update] method
    pub fn show(&mut self, ui: &mut Ui, data: &mut AppData, icons: Arc<IconTextures>) {
        if self.state.is_close() {
            return;
        }

        ui.vertical_centered(|ui| {
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                self.available_wallets(ui, data);

                // show the balance of the selected wallet
                let owner = data.wallet_address();
                let (_, balance) = data.eth_balance(data.chain_id.id(), owner);
                let formated = format!("{:.4}", format_ether(balance));
                let balance_text = RichText::new(&formated)
                    .family(roboto_regular())
                    .size(15.0)
                    .color(Color32::WHITE);

                ui.add(icons.currency_icon(data.chain_id.id()));
                ui.label(balance_text);
            });
            // TODO: Portofolio value in USD
        });
    }

    fn available_wallets(&self, ui: &mut Ui, data: &mut AppData) {
        let wallet_name = &data.profile.current_wallet_name_truncated();
        let selected_text = RichText::new(wallet_name)
            .family(roboto_regular())
            .size(13.0)
            .color(Color32::WHITE);

        ComboBox::from_label("")
            .selected_text(selected_text)
            .width(30.0)
            .height(5.0)
            .show_ui(ui, |ui| {
                for wallet in &data.profile.wallets {
                    ui.selectable_value(
                        &mut data.profile.current_wallet,
                        Some(wallet.clone()),
                        wallet.name.clone(),
                    );
                }
            });
    }
}
