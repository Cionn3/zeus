use eframe::{
    egui::{Align2, Button, ComboBox, Sense, Ui, Window},
    epaint::vec2,
};
use std::{collections::HashMap, sync::Arc};

use super::{
    icons::IconTextures,
    misc::{frame, rich_text, button, text_edit_s},
    GUI,
};
use zeus_backend::types::Request;
use zeus_chain::alloy::primitives::utils::format_ether;
use zeus_shared_types::{AppData, ErrorMsg, SHARED_UI_STATE};


/// Paint the UI repsonsible for managing the wallets
pub fn wallet_ui(ui: &mut Ui, data: &mut AppData, gui: &GUI) {
    wallet_selection(ui, data, gui.icons.clone());

    new_wallet(ui);
    generate_new_wallet(ui, data, gui);
    import_wallet(ui, data, gui);

    export_key_ui(ui, data);

    show_exported_key(ui);
}

fn wallet_selection(ui: &mut Ui, data: &mut AppData, icons: Arc<IconTextures>) {
    if !data.logged_in || data.new_profile_screen {
        return;
    }

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            frame().show(ui, |ui| {
                let balance = data.eth_balance(data.chain_id.id());
                let formated = format!("{} {:.4}", data.native_coin(), format_ether(balance));
                let balance_text = rich_text(&formated, 15.0);

                let selected_text = rich_text(&data.profile.current_wallet_name(), 13.0);

                let wallets_text = rich_text("Wallets", 13.0);

                ui.label(wallets_text);
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
                ui.label(balance_text);
            });
        });

        ui.horizontal(|ui| {
            ui.set_min_size(vec2(200.0, 50.0));

            ui.add_space(10.0);

            // New Wallet Icon, If clicked, open the [new_wallet] UI
            let wallet_new_res = ui.add(icons.wallet_new_icon());
            if wallet_new_res.clicked() {
                let mut state = SHARED_UI_STATE.write().unwrap();
                state.new_wallet_window_on = true;
            }

            ui.add_space(10.0);

            // copy current wallet address to clipboard
            let copy_addr_res = ui.add(icons.copy_icon());
            if copy_addr_res.clicked() {
                let curr_wallet = data.profile.current_wallet.clone();

                let curr_wallet = match curr_wallet {
                    Some(wallet) => wallet,
                    None => {
                        let mut state = SHARED_UI_STATE.write().unwrap();
                        state.err_msg = ErrorMsg::new(true, "No Wallet Selected");
                        return;
                    }
                };

                ui.ctx().output_mut(|output| {
                    output.copied_text = curr_wallet.key.address().to_string();
                });
            }

            ui.add_space(5.0);

            let export_key_res = ui.add(icons.export_key_icon());

            if export_key_res.clicked() {
                let mut state = SHARED_UI_STATE.write().unwrap();
                state.export_key_ui = true;
            }
        });
        ui.add_space(10.0);
    });
}

/// Prompt the user to create a new random wallet or import one
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

/// Generate a new wallet UI
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
                            state.err_msg = ErrorMsg::new(true, e);
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

/// Import Wallet UI
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
                            state.err_msg = ErrorMsg::new(true, e);
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
                let username_field = text_edit_s(data.confirm_credentials.user_mut(), 200.0, false);
                ui.label(username);
                ui.add(username_field);
                ui.add_space(10.0);
            }

            {
                let password_field = text_edit_s(data.confirm_credentials.passwd_mut(), 200.0, true);
                ui.label(password);
                ui.add(password_field);
                ui.add_space(10.0);
            }

            {
                let confirm_field = text_edit_s(
                    data.confirm_credentials.confirm_passwd_mut(),
                    200.0,
                    true,
                );
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
                        state.err_msg = ErrorMsg::new(true, "No Wallet Selected");
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
                        state.err_msg = ErrorMsg::new(true, e);
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
