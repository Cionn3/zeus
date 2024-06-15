use eframe::{
    egui::{
        Color32,
        Frame,
        Ui,
        RichText,
        widgets::TextEdit,
        Rounding,
        vec2
    },
    epaint::{ Margin, Shadow },
    emath::Vec2,
};

use crate::{ fonts::roboto_regular, ZeusApp };

use super::THEME;


/// Render the login screen
pub fn login_screen(ui: &mut Ui, app: &mut ZeusApp) {
    
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
                &mut app.data.profile.credentials.username
            ).desired_width(150.0);

            let pass_field = TextEdit::singleline(&mut app.data.profile.credentials.password)
                .desired_width(150.0)
                .password(true);

            let confrim_field = TextEdit::singleline(&mut app.data.profile.credentials.confrim_password).desired_width(150.0).password(true);

            
            ui.vertical_centered(|ui| {
                ui.label(user_text);
                ui.add(user_field);

                ui.add_space(10.0);

                ui.label(pass_text);
                ui.add(pass_field);
                ui.add_space(15.0);

                ui.label(confrim_text);
                ui.add(confrim_field);
                ui.add_space(15.0);
            });

            ui.horizontal(|ui| {
                // TODO: dont center the buttons manually
                ui.add_space(120.0);
                if ui.button("Unlock").clicked() {

                    match app.data.profile.decrypt_and_load() {
                        Ok(_) => {
                            app.data.logged_in = true;
                        }
                        Err(e) => {
                            println!("Failed to unlock profile: {:?}", e);
                        }
                    }
                }           
            });
        });
    });
}


/// Render the new profile screen
pub fn new_profile_screen(ui: &mut Ui, app: &mut ZeusApp) {
    if !app.data.new_profile_screen {
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
                &mut app.data.profile.credentials.username
            ).desired_width(150.0);

            let pass_field = TextEdit::singleline(&mut app.data.profile.credentials.password)
                .desired_width(150.0)
                .password(true);

            let confirm_field = TextEdit::singleline(
                &mut app.data.profile.credentials.confrim_password
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
        });

        ui.horizontal(|ui| {
            ui.add_space(150.0);
            if ui.button("Create Profile").clicked() {
                
                // encrypt and save the wallets to disk
                match app.data.profile.encrypt_and_save() {
                    Ok(_) => {
                        println!("Profile Created");
                    }
                    Err(e) => {
                        println!("Failed to create profile: {:?}", e);
                    }
                }
                

                app.data.new_profile_screen = false;
                app.data.profile_exists = true;
                app.data.logged_in = true;
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

