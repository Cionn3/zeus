use eframe::{
    egui::{
        vec2, widgets::TextEdit, Align2, Button, Checkbox, Color32, Frame, RichText, Rounding, Ui, Window
    }, emath::Vec2, epaint::{ Margin, Shadow }
};

use crate::fonts::roboto_regular;

use super::THEME;
use super::super::app::ZeusApp;
use zeus_shared_types::{AppData, ErrorMsg, SHARED_UI_STATE};

use tracing::trace;

/// Paint the login area
pub fn paint_login(ui: &mut Ui, data: &mut AppData) {
    // profile found but not logged in
    if data.profile_exists && !data.logged_in {
        login_screen(ui, data);
    }

    // if this is true then the user has not created a profile yet
    if data.new_profile_screen {
        new_profile_screen(ui, data);
    }
}


/// Paint the login screen
pub fn login_screen(ui: &mut Ui, data: &mut AppData) {
    
    frame().show(ui, |ui| {
        ui.set_max_size(Vec2::new(400.0, 500.0));

        let heading = rich_text("Unlock Profile", 16.0);
        ui.label(heading);
        ui.add_space(30.0);

        ui.vertical_centered(|ui| {

            let user_text = rich_text("Username", 16.0);

            let pass_text = rich_text("Password", 16.0);

            let confrim_text = rich_text("Confirm Password", 16.0);

            let user_field = TextEdit::singleline(
                &mut data.profile.credentials.username
            ).desired_width(150.0);

            let pass_field = TextEdit::singleline(&mut data.profile.credentials.password)
                .desired_width(150.0)
                .password(true);

            let confrim_field = TextEdit::singleline(&mut data.profile.credentials.confrim_password).desired_width(150.0).password(true);

            
            ui.vertical_centered(|ui| {
                ui.label(user_text);
                ui.add(user_field);

                ui.add_space(10.0);

                ui.label(pass_text);
                ui.add(pass_field);
                ui.add_space(10.0);

                ui.label(confrim_text);
                ui.add(confrim_field);
                ui.add_space(15.0);
            });
             
                let unlock_txt = rich_text("Unlock", 15.0);
                if ui.button(unlock_txt).clicked() {

                    match data.profile.decrypt_and_load() {
                        Ok(_) => {
                            trace!("Profile unlocked");
                            data.logged_in = true;
                        }
                        Err(e) => {
                            let mut state = SHARED_UI_STATE.write().unwrap();
                            state.err_msg = ErrorMsg::new(true, e);
                        }
                    }
                }           
           
        });
    });
}


/// Paint the new profile screen
pub fn new_profile_screen(ui: &mut Ui, data: &mut AppData) {
    if !data.new_profile_screen {
        return;
    }

    
    frame().show(ui, |ui| {
        ui.set_max_size(Vec2::new(400.0, 500.0));

        let heading = rich_text("Create Profile", 16.0);

        ui.label(heading);
        ui.add_space(30.0);

        ui.vertical_centered(|ui| {

            let user_text = rich_text("Username", 16.0);

            let pass_text = rich_text("Password", 16.0);

            let confirm_text = rich_text("Confirm Password", 16.0);

            let user_field = TextEdit::singleline(
                &mut data.profile.credentials.username
            ).desired_width(150.0);

            let pass_field = TextEdit::singleline(&mut data.profile.credentials.password)
                .desired_width(150.0)
                .password(true);

            let confirm_field = TextEdit::singleline(
                &mut data.profile.credentials.confrim_password
            )
                .desired_width(150.0)
                .password(true);

            ui.label(user_text);
            ui.add(user_field);

            ui.add_space(10.0);

            ui.label(pass_text);
            ui.add(pass_field);

            ui.add_space(10.0);

            ui.label(confirm_text);
            ui.add(confirm_field);

            ui.add_space(15.0);
        

        
            let create_txt = rich_text("Create", 15.0);
            let button = Button::new(create_txt).rounding(10.0).min_size(vec2(30.0, 10.0));
            let res = ui.add(button);

            if res.clicked() {   
                // encrypt and save the wallets to disk
                match data.profile.encrypt_and_save() {
                    Ok(_) => {
                        data.new_profile_screen = false;
                        data.profile_exists = true;
                        data.logged_in = true;
                    }
                    Err(e) => {
                        let mut state = SHARED_UI_STATE.write().unwrap();
                        state.err_msg = ErrorMsg::new(true, e);
                    }
                }

            }
        });
    });
}


/// TxSettings popup
pub fn tx_settings_window(ui: &mut Ui, data: &mut AppData) {
    {
    let state = SHARED_UI_STATE.read().unwrap();
    if !state.tx_settings_on {
        return;
    }
}

    Window::new("Transaction Settings")
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .collapsible(false)
        .show(ui.ctx(), |ui| {
            ui.set_max_size(vec2(200.0, 100.0));

            ui.vertical_centered(|ui| {
                let priority_fee = rich_text("Priority Fee (Gwei)", 15.0);
                let slippage_text = rich_text("Slippage", 15.0);
                let mev_protect = rich_text("MEV Protect", 15.0);

                let fee_field = TextEdit::singleline(&mut data.tx_settings.priority_fee)
                    .desired_width(15.0);

                let slippage_field = TextEdit::singleline(&mut data.tx_settings.slippage)
                    .desired_width(15.0);

                let mev_protect_check = Checkbox::new(&mut data.tx_settings.mev_protect, "");

                ui.horizontal(|ui| {
                    ui.label(priority_fee);
                    ui.add_space(5.0);
                    ui.add(fee_field);
                   
                });
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label(slippage_text);
                    ui.add_space(5.0);
                    ui.add(slippage_field);
                });
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label(mev_protect);
                    ui.add_space(5.0);
                    ui.add(mev_protect_check);
                });
                ui.add_space(10.0);

                if ui.button("Save").clicked() {
                    // TODO save the settings
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.tx_settings_on = false;
                }
            });
        });
            
}

    /// Show an error message if needed
    pub fn err_msg(ui: &mut Ui) {
        let err_msg;
        {
            let state = SHARED_UI_STATE.read().unwrap();
            err_msg = state.err_msg.msg.clone();
            if !state.err_msg.on {
                return;
            }
        }

        Window::new("Error")
            .resizable(false)
            .anchor(Align2::CENTER_TOP, vec2(0.0, 0.0))
            .collapsible(false)
            .title_bar(false)
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    let msg_text = rich_text(&err_msg, 16.0);
                    let close_text = rich_text("Close", 16.0);

                    ui.label(msg_text);
                    ui.add_space(5.0);
                    if ui.button(close_text).clicked() {
                        let mut state = SHARED_UI_STATE.write().unwrap();
                        state.err_msg.on = false;
                    }
                });
            });
    }


    // TODO: Auto close it after a few seconds
    /// Show an info message if needed
    pub fn info_msg(ui: &mut Ui) {
        {
            let state = SHARED_UI_STATE.read().unwrap();
            if !state.info_msg.on {
                return;
            }
        }

        ui.vertical_centered_justified(|ui| {
            frame().show(ui, |ui| {
                ui.set_max_size(vec2(1000.0, 50.0));

                let info_msg;
                {
                    let state = SHARED_UI_STATE.read().unwrap();
                    info_msg = state.info_msg.msg.clone();
                }
                let msg_text = rich_text(&info_msg, 16.0);
                let close_text = rich_text("Close", 16.0);

                ui.label(msg_text);
                ui.add_space(5.0);
                if ui.button(close_text).clicked() {
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.info_msg.on = false;
                }
            });
        });
    }



/// Returns a [Frame] that is commonly used
pub fn frame() -> Frame {
    Frame {
        inner_margin: Margin::same(8.0),
        outer_margin: Margin::same(8.0),
        fill: THEME.colors.darker_gray,
        rounding: Rounding { ne: 8.0, se: 8.0, sw: 8.0, nw: 8.0 },
        shadow: Shadow {
            offset: vec2(0.0, 0.0),
            blur: 4.0,
            spread: 0.0,
            color: Color32::from_gray(128),
        },
        ..Frame::default()
    }
}


/// Returns a [RichText] that is commonly used
/// 
/// Shortcut for `RichText::new("text").family(roboto_regular()).size(f32).color(THEME.colors.white)`
pub fn rich_text(text: &str, size: f32) -> RichText {
    RichText::new(text)
        .family(roboto_regular())
        .size(size)
        .color(THEME.colors.white)
}

