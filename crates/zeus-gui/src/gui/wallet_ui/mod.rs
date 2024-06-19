use eframe::{
    egui::{
        Ui, TextEdit,
        Align2, ComboBox
    },
    epaint::vec2,
};

use super::{misc::{frame, rich_text}, GUI};
use crate::AppData;

use zeus_backend::types::Request;

use crate::gui::{state::*, THEME};



    /// Render the UI repsonsible for managing the wallets
    pub fn wallet_ui (ui: &mut Ui, data: &mut AppData, shared_state: &mut SharedUiState, gui: &GUI) {
        wallet_selection(ui, data, shared_state);
        create_or_import_wallet(ui, data, shared_state, gui);
        export_key_ui(ui, data, shared_state);
        show_exported_key(ui, shared_state);

    }

    fn wallet_selection(ui: &mut Ui, data: &mut AppData, shared_state: &mut SharedUiState) {
         
        if !data.logged_in || data.new_profile_screen {
            return;
        }

        ui.vertical(|ui| {
            ui.horizontal(|ui| {

                frame().show(ui, |ui| {
                    let current_wallet = data.profile.current_wallet_name();

                    // TODO: an oracle to fetch the balance at specific intervals
                    let balance = rich_text(&data.native_balance(), 15.0);

                    let coin = rich_text(&data.native_coin(), 15.0);

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
                    ui.label(coin);
                    ui.label(balance);
                });
            });

            ui.horizontal(|ui| {

            
            ui.vertical(|ui| {
                if ui.button("New Wallet").clicked() {
                    shared_state.wallet_popup = (true, "New");
                }

                ui.add_space(5.0);

                if ui.button("Import Wallet").clicked() {
                    shared_state.wallet_popup = (true, "Import Wallet");
                }
            });

            ui.vertical(|ui| {
                if ui.button("Copy Address").clicked() {

                    let curr_wallet = data.profile.current_wallet.clone();

                    let curr_wallet = match curr_wallet {
                        Some(wallet) => wallet,
                        None => {
                            shared_state.err_msg = ErrorMsg::new(true, "No Wallet Selected");
                            return;
                        }
                    };
                    
                    ui.ctx().output_mut(|output| {
                        output.copied_text = curr_wallet.key.address().to_string();
                    });
                }

                ui.add_space(5.0);

                if ui.button("Export Private Key").clicked() {
                    shared_state.export_key_ui = true;
                }
            });


        });
            ui.add_space(10.0);

        });
    }


    fn create_or_import_wallet(ui: &mut Ui, data: &mut AppData, shared_state: &mut SharedUiState, gui: &GUI) {

        if !shared_state.wallet_popup.0 {
            return;
        }

        egui::Window::new(shared_state.wallet_popup.1)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .collapsible(false)
        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {

           if shared_state.wallet_popup.1 == "Import Wallet" {

            let label = rich_text("Private key:", 15.0);
            let name_label = rich_text("Wallet Name (Optional):", 15.0);

            let key_field = TextEdit::singleline(&mut data.private_key).desired_width(200.0).password(true);
            let name_field = TextEdit::singleline(&mut data.wallet_name).desired_width(200.0);

            
                ui.label(label);
                ui.add_space(5.0);
                ui.add(key_field);
                ui.add_space(5.0);
                ui.label(name_label);
                ui.add_space(5.0);
                ui.add(name_field);
                ui.add_space(5.0);

               if ui.button("Import").clicked() {

                    match data.profile.import_wallet(data.wallet_name.clone(), data.private_key.clone()) {
                        Ok(_) => {}
                        Err(e) => {
                            shared_state.err_msg = ErrorMsg::new(true, e);
                        }
                    }

                    shared_state.wallet_popup = (false, "Import Wallet");
                    data.private_key = "".to_string();
                    data.wallet_name = "".to_string();
                    gui.send_request(Request::SaveProfile { profile: data.profile.clone() }, shared_state);
                    
                }
                if ui.button("Close").clicked() {
                    shared_state.wallet_popup = (false, "Import Wallet");
                }
               
            } else if shared_state.wallet_popup.1 == "New" {
                let label = rich_text("Wallet Name (Optional):", 15.0);
                let name_field = TextEdit::singleline(&mut data.wallet_name).desired_width(200.0);

                ui.label(label);
                ui.add_space(5.0);
                ui.add(name_field);
                ui.add_space(5.0);

                if ui.button("Create").clicked() {
                    data.profile.new_wallet(data.wallet_name.clone());
                    shared_state.wallet_popup = (false, "New");
                    data.wallet_name = "".to_string();
                    gui.send_request(Request::SaveProfile { profile: data.profile.clone() }, shared_state);
                }
                if ui.button("Close").clicked() {
                    shared_state.wallet_popup = (false, "New");
                }
            }
            });
        });
        
    }


    pub fn export_key_ui(ui: &mut Ui, data: &mut AppData, shared_state: &mut SharedUiState) {
        if !shared_state.export_key_ui {
            return;
        }

        let heading = rich_text("Confirm Your Credentials", 20.0);
        let username = rich_text("Username", 15.0);
        let password = rich_text("Password", 15.0);
        let confirm_password = rich_text("Confirm Password", 15.0);

        egui::Window::new("Export Key")
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .collapsible(false)
        .show(ui.ctx(), |ui| {
            ui.label(heading);
            ui.add_space(10.0);

            ui.label(username);
            let username_field = TextEdit::singleline(&mut data.confirm_credentials.username).text_color(THEME.colors.dark_gray);
            ui.add(username_field);
            ui.add_space(10.0);

            ui.label(password);
            let password_field = TextEdit::singleline(&mut data.confirm_credentials.password).password(true);
            ui.add(password_field);
            ui.add_space(10.0);

            ui.label(confirm_password);
            let confirm_password_field = TextEdit::singleline(&mut data.confirm_credentials.confrim_password).password(true);
            ui.add(confirm_password_field);
            ui.add_space(10.0);

            if ui.button("Export Key").clicked() {
                let wallet = match data.profile.current_wallet.clone() {
                    Some(wallet) => wallet,
                    None => {
                        shared_state.err_msg = ErrorMsg::new(true, "No Wallet Selected");
                        return;
                    }
                };

                let key = match data.profile.export_wallet(wallet.name, data.confirm_credentials.clone()) {
                    Ok(key) => key,
                    Err(e) => {
                        shared_state.err_msg = ErrorMsg::new(true, e);
                        return;
                    }
                };
                data.confirm_credentials = Default::default();

                shared_state.export_key_ui = false;
                shared_state.exported_key_window = (true, key);
            }
            if ui.button("Close").clicked() {
                shared_state.export_key_ui = false;
            }
        });
    }



    pub fn show_exported_key(ui: &mut Ui, shared_state: &mut SharedUiState) {
        if !shared_state.exported_key_window.0 {
            return;
        }

        egui::Window::new("Exported Key")
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .collapsible(false)
        .show(ui.ctx(), |ui| {
        let label = rich_text(&shared_state.exported_key_window.1, 15.0);
        ui.label(label);

        if ui.button("Close").clicked() {
            shared_state.exported_key_window = (false, "".to_string());
        }

        });
    }