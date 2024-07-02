use eframe::{ egui::{ Ui, TextEdit, Align2, ComboBox, Window }, epaint::vec2 };

use super::{ misc::{ frame, rich_text }, GUI };
use zeus_backend::types::Request;
use zeus_shared_types::{AppData, ErrorMsg, SHARED_UI_STATE};
use zeus_chain::alloy::primitives::utils::format_ether;

use crate::gui::THEME;

/// Render the UI repsonsible for managing the wallets
pub fn wallet_ui(ui: &mut Ui, data: &mut AppData, gui: &GUI) {
    wallet_selection(ui, data);
    create_or_import_wallet(ui, data, gui);
    export_key_ui(ui, data);
    show_exported_key(ui);
}

fn wallet_selection(ui: &mut Ui, data: &mut AppData) {
    
        if !data.logged_in || data.new_profile_screen {
            return;
        }
    

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            frame().show(ui, |ui| {
                let current_wallet = data.profile.current_wallet_name();
                let balance = data.eth_balance(data.chain_id.id());
                let formated = format!("{} {:.4}",data.native_coin(), format_ether(balance));
                let balance_text = rich_text(&formated, 15.0);

                ComboBox::from_label("")
                    .selected_text(current_wallet)
                    .show_ui(ui, |ui| {
                        for wallet in &data.profile.wallets {
                            ui.selectable_value(
                                &mut data.profile.current_wallet,
                                Some(wallet.clone()),
                                wallet.name.clone()
                            );
                        }
                    });
                ui.label(balance_text);
            });
        });

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                if ui.button("New Wallet").clicked() {
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.wallet_popup = (true, "New");
                }

                ui.add_space(5.0);

                if ui.button("Import Wallet").clicked() {
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.wallet_popup = (true, "Import Wallet");
                }
            });

            ui.vertical(|ui| {
                if ui.button("Copy Address").clicked() {
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

                if ui.button("Export Private Key").clicked() {
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.export_key_ui = true;
                }
            });
        });
        ui.add_space(10.0);
    });
}

fn create_or_import_wallet(ui: &mut Ui, data: &mut AppData, gui: &GUI) {
    let wallet_action;
    {
        let state = SHARED_UI_STATE.read().unwrap();
        wallet_action = state.wallet_popup.1;
        if !state.wallet_popup.0 {
            return;
        }
    }

    Window
        ::new(wallet_action)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .collapsible(false)
        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                if wallet_action == "Import Wallet" {
                    let label = rich_text("Private key:", 15.0);
                    let name_label = rich_text("Wallet Name (Optional):", 15.0);

                    let key_field = TextEdit::singleline(&mut data.private_key)
                        .desired_width(200.0)
                        .password(true);
                    let name_field = TextEdit::singleline(&mut data.wallet_name).desired_width(
                        200.0
                    );

                    ui.label(label);
                    ui.add_space(5.0);
                    ui.add(key_field);
                    ui.add_space(5.0);
                    ui.label(name_label);
                    ui.add_space(5.0);
                    ui.add(name_field);
                    ui.add_space(5.0);

                    if ui.button("Import").clicked() {
                        match
                            data.profile.import_wallet(
                                data.wallet_name.clone(),
                                data.private_key.clone()
                            )
                        {
                            Ok(_) => {}
                            Err(e) => {
                                let mut state = SHARED_UI_STATE.write().unwrap();
                                state.err_msg = ErrorMsg::new(true, e);
                            }
                        }

                        let mut state = SHARED_UI_STATE.write().unwrap();
                        state.wallet_popup = (false, "Import Wallet");
                        data.private_key = "".to_string();
                        data.wallet_name = "".to_string();
                        gui.send_request(Request::SaveProfile { profile: data.profile.clone() });
                    }
                    if ui.button("Close").clicked() {
                        let mut state = SHARED_UI_STATE.write().unwrap();
                        state.wallet_popup = (false, "Import Wallet");
                    }
                } else if wallet_action == "New" {
                    let label = rich_text("Wallet Name (Optional):", 15.0);
                    let name_field = TextEdit::singleline(&mut data.wallet_name).desired_width(
                        200.0
                    );

                    ui.label(label);
                    ui.add_space(5.0);
                    ui.add(name_field);
                    ui.add_space(5.0);

                    if ui.button("Create").clicked() {
                        data.profile.new_wallet(data.wallet_name.clone());
                        let mut state = SHARED_UI_STATE.write().unwrap();
                        state.wallet_popup = (false, "New");
                        data.wallet_name = "".to_string();
                        gui.send_request(Request::SaveProfile { profile: data.profile.clone() });
                    }
                    if ui.button("Close").clicked() {
                        let mut state = SHARED_UI_STATE.write().unwrap();
                        state.wallet_popup = (false, "New");
                    }
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

    Window
        ::new("Export Key")
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .collapsible(false)
        .show(ui.ctx(), |ui| {
            ui.label(heading);
            ui.add_space(10.0);

            ui.label(username);
            let username_field = TextEdit::singleline(
                &mut data.confirm_credentials.username
            ).text_color(THEME.colors.dark_gray);
            ui.add(username_field);
            ui.add_space(10.0);

            ui.label(password);
            let password_field = TextEdit::singleline(
                &mut data.confirm_credentials.password
            ).password(true);
            ui.add(password_field);
            ui.add_space(10.0);

            ui.label(confirm_password);
            let confirm_password_field = TextEdit::singleline(
                &mut data.confirm_credentials.confrim_password
            ).password(true);
            ui.add(confirm_password_field);
            ui.add_space(10.0);

            if ui.button("Export Key").clicked() {
                let wallet = match data.profile.current_wallet.clone() {
                    Some(wallet) => wallet,
                    None => {
                        let mut state = SHARED_UI_STATE.write().unwrap();
                        state.err_msg = ErrorMsg::new(true, "No Wallet Selected");
                        return;
                    }
                };

                let key = match
                    data.profile.export_wallet(wallet.name, data.confirm_credentials.clone())
                {
                    Ok(key) => key,
                    Err(e) => {
                        let mut state = SHARED_UI_STATE.write().unwrap();
                        state.err_msg = ErrorMsg::new(true, e);
                        return;
                    }
                };
                data.confirm_credentials = Default::default();

                let mut state = SHARED_UI_STATE.write().unwrap();
                state.export_key_ui = false;
                state.exported_key_window = (true, key);
            }
            if ui.button("Close").clicked() {
                let mut state = SHARED_UI_STATE.write().unwrap();
                state.export_key_ui = false;
            }
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

    Window
        ::new("Exported Key")
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
