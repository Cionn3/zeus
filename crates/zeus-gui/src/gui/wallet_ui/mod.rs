use eframe::{
    egui::{Align2, Button, ComboBox, Sense, Ui, Window},
    epaint::vec2,
};
use std::{collections::HashMap, sync::Arc};


use super::{
    super::icons::IconTextures,
    misc::{button, rich_text, text_edit_s},
    GUI,
};
use zeus_backend::types::Request;
use zeus_chain::alloy::primitives::utils::format_ether;
use zeus_shared_types::{AppData, SHARED_UI_STATE};

/// Paint the UI repsonsible for managing the wallets
pub fn wallet_ui(ui: &mut Ui, data: &mut AppData, gui: &GUI) {
    wallet_selection(ui, data, gui.theme.icons.clone());

    new_wallet(ui);
    generate_new_wallet(ui, data, gui);
    import_wallet(ui, data, gui);

    export_key_ui(ui, data);

    show_exported_key(ui);
}

/// Wallet Selection UI
/// 
/// This UI is responsible for displaying the available wallets and the balance of the selected wallet
fn wallet_selection(ui: &mut Ui, data: &mut AppData, icons: Arc<IconTextures>) {
    if !data.logged_in || data.new_profile_screen {
        return;
    }

    ui.vertical_centered(|ui| {
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            // show the available walletss
            available_wallets(ui, data);

            // show the balance of the selected wallet
            let owner = data.wallet_address();
            let (_, balance) = data.eth_balance(data.chain_id.id(), owner);
            let formated = format!("{:.4}", format_ether(balance));
            let balance_text = rich_text(&formated, 15.0);

            ui.add(icons.currency_icon(data.chain_id.id()));
            ui.label(balance_text);
        });
        // TODO: Portofolio value in USD
    });
}

/// Show the available wallets
fn available_wallets(ui: &mut Ui, data: &mut AppData) {
    let wallet_name = &data.profile.current_wallet_name();
    let selected_text = rich_text(wallet_name, 13.0);

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



/// Prompt the user to create a new random wallet or import one
/// 
/// Depends on the state of the [SHARED_UI_STATE]
fn new_wallet(ui: &mut Ui) {
    {
        let state = SHARED_UI_STATE.read().unwrap();
        if !state.new_wallet_window_on {
            return;
        }
    }

    Window::new("New Wallet")
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .collapsible(false)
        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                let generate_text = rich_text("Generate a new one", 15.0);
                let generate_button = button(generate_text);

                ui.add_space(30.0);
                let generate_res = ui.add(generate_button);

                let import_text = rich_text("Import from private key", 15.0);
                let import_button = button(import_text);

                ui.add_space(30.0);
                let import_res = ui.add(import_button);

                let close_text = rich_text("Close", 15.0);
                let close_button = button(close_text);

                ui.add_space(30.0);
                let close_res = ui.add(close_button);

                if generate_res.clicked() {
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.generate_wallet_on = true;
                    state.new_wallet_window_on = false;
                }

                if import_res.clicked() {
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.import_wallet_window_on = true;
                    state.new_wallet_window_on = false;
                }

                if close_res.clicked() {
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.new_wallet_window_on = false;
                }
            });
        });
}

/// This UI is responsible for generating a new wallet
/// 
/// Depends on the state of the [SHARED_UI_STATE]
fn generate_new_wallet(ui: &mut Ui, data: &mut AppData, gui: &GUI) {
    {
        let state = SHARED_UI_STATE.read().unwrap();
        if !state.generate_wallet_on {
            return;
        }
    }

    Window::new("New Wallet")
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .collapsible(false)
        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                let label = rich_text("Wallet Name (Optional):", 15.0);
                let name_field = text_edit_s(&mut data.wallet_name, 200.0, false);

                ui.label(label);
                ui.add_space(5.0);
                ui.add(name_field);
                ui.add_space(5.0);

                let create_text = rich_text("Create", 15.0);
                let create_button = Button::new(create_text)
                    .rounding(10.0)
                    .sense(Sense::click());

                let close_text = rich_text("Close", 15.0);
                let close_button = Button::new(close_text).rounding(10.0).sense(Sense::click());

                let create_res = ui.add(create_button);

                ui.add_space(15.0);
                let close_res = ui.add(close_button);

                if create_res.clicked() {
                    match data.profile.new_wallet(data.wallet_name.clone()) {
                        Ok(_) => {}
                        Err(e) => {
                            let mut state = SHARED_UI_STATE.write().unwrap();
                            state.err_msg.show(e);
                        }
                    }

                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.generate_wallet_on = false;
                    data.wallet_name = "".to_string();

                    gui.send_request(Request::SaveProfile {
                        profile: data.profile.clone(),
                    });
                }

                if close_res.clicked() {
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.generate_wallet_on = false;
                }
            });
        });
}


/// This UI is responsible for importing a wallet from a private key
/// 
/// Depends on the state of the [SHARED_UI_STATE]
fn import_wallet(ui: &mut Ui, data: &mut AppData, gui: &GUI) {
    {
        let state = SHARED_UI_STATE.read().unwrap();
        if !state.import_wallet_window_on {
            return;
        }
    }

    Window::new("Import Wallet")
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .collapsible(false)
        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                let label = rich_text("Private key:", 15.0);
                let name_label = rich_text("Wallet Name (Optional):", 15.0);

                let key_field = text_edit_s(&mut data.private_key, 200.0, true);
                let name_field = text_edit_s(&mut data.wallet_name, 200.0, false);

                let import_text = rich_text("Import", 15.0);
                let import_button = Button::new(import_text)
                    .rounding(10.0)
                    .sense(Sense::click());

                let close_text = rich_text("Close", 15.0);
                let close_button = Button::new(close_text).rounding(10.0).sense(Sense::click());

                ui.label(label);
                ui.add_space(10.0);
                ui.add(key_field);
                ui.add_space(10.0);
                ui.label(name_label);
                ui.add_space(10.0);
                ui.add(name_field);
                ui.add_space(10.0);

                let import_res = ui.add(import_button);
                ui.add_space(10.0);

                let close_res = ui.add(close_button);

                if import_res.clicked() {
                    match data.profile.import_wallet(
                        data.wallet_name.clone(),
                        HashMap::new(),
                        data.private_key.clone(),
                    ) {
                        Ok(_) => {}
                        Err(e) => {
                            let mut state = SHARED_UI_STATE.write().unwrap();
                            state.err_msg.show(e);
                        }
                    }

                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.import_wallet_window_on = false;

                    // clear the fields
                    data.private_key.clear();
                    data.wallet_name.clear();

                    gui.send_request(Request::SaveProfile {
                        profile: data.profile.clone(),
                    });
                }

                if close_res.clicked() {
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.import_wallet_window_on = false;
                }
            });
        });
}

/// This UI is responsible for exporting the private key of a wallet
/// 
/// Depends on the state of the [SHARED_UI_STATE]
pub fn export_key_ui(ui: &mut Ui, data: &mut AppData) {
    {
        let state = SHARED_UI_STATE.read().unwrap();
        if !state.export_key_ui {
            return;
        }
    }

    let heading = rich_text("Confirm Your Credentials", 20.0);
    let username = rich_text("Username", 15.0);
    let password = rich_text("Password", 15.0);
    let confirm_password = rich_text("Confirm Password", 15.0);

    Window::new("Export Key")
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .collapsible(false)
        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.label(heading);
                ui.add_space(10.0);

                {
                    let username_field =
                        text_edit_s(data.confirm_credentials.user_mut(), 200.0, false);
                    ui.label(username);
                    ui.add(username_field);
                    ui.add_space(10.0);
                }

                {
                    let password_field =
                        text_edit_s(data.confirm_credentials.passwd_mut(), 200.0, true);
                    ui.label(password);
                    ui.add(password_field);
                    ui.add_space(10.0);
                }

                {
                    let confirm_field =
                        text_edit_s(data.confirm_credentials.confirm_passwd_mut(), 200.0, true);
                    ui.label(confirm_password);
                    ui.add(confirm_field);
                }

                ui.add_space(10.0);

                let export_text = rich_text("Export Key", 15.0);
                let export_button = button(export_text);

                if ui.add(export_button).clicked() {
                    let wallet = match data.profile.current_wallet.clone() {
                        Some(wallet) => wallet,
                        None => {
                            let mut state = SHARED_UI_STATE.write().unwrap();
                            state.err_msg.show("No wallet selected");
                            return;
                        }
                    };

                    let key = match data
                        .profile
                        .export_wallet(wallet.name, data.confirm_credentials.clone())
                    {
                        Ok(key) => key,
                        Err(e) => {
                            let mut state = SHARED_UI_STATE.write().unwrap();
                            state.err_msg.show(e);
                            return;
                        }
                    };

                    // clear the confirm credentials
                    data.confirm_credentials.clear();

                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.export_key_ui = false;
                    state.exported_key_window = (true, key);
                }
                ui.add_space(10.0);

                let close_text = rich_text("Close", 15.0);
                let close_button = button(close_text);

                if ui.add(close_button).clicked() {
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.export_key_ui = false;
                }
            });
        });
}

/// Show the exported key
/// 
/// Depends on the state of the [SHARED_UI_STATE]
pub fn show_exported_key(ui: &mut Ui) {
    let window_text;
    {
        let state = SHARED_UI_STATE.read().unwrap();
        window_text = state.exported_key_window.1.clone();
        if !state.exported_key_window.0 {
            return;
        }
    }

    Window::new("Exported Key")
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .collapsible(false)
        .show(ui.ctx(), |ui| {
            let label = rich_text(&window_text, 15.0);
            ui.label(label);

            if ui.button("Close").clicked() {
                let mut state = SHARED_UI_STATE.write().unwrap();
                state.exported_key_window = (false, "".to_string());
            }
        });
}
