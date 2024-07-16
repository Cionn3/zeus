use crate::{fonts::roboto_regular, icons::IconTextures};
use eframe::egui::{
    Align, Align2, Button, Color32, FontId, Layout, RichText, Sense, TextEdit, Ui,
    Vec2, Window,
};
use std::sync::Arc;
use zeus_shared_types::{AppData, ErrorMsg, SHARED_UI_STATE};

use tracing::{error, trace};

    /// The Network Settings UI
    pub fn networks_settings_ui(
        ui: &mut Ui,
        data: &mut AppData,
        icons: Arc<IconTextures>,
    ) {
        {
            let state = SHARED_UI_STATE.read().unwrap();
            if !state.network_settings {
                return;
            }
        }

        let font = FontId::new(15.0, roboto_regular());
        let save = RichText::new("Save")
            .family(roboto_regular())
            .size(15.0)
            .color(Color32::WHITE);

        let settings = RichText::new("Network Settings")
            .family(roboto_regular())
            .size(20.0)
            .color(Color32::WHITE);

        let save_button = Button::new(save)
            .rounding(10.0)
            .sense(Sense::click())
            .min_size(Vec2::new(70.0, 25.0));

        Window::new(settings)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.set_min_size(Vec2::new(250.0, 320.0));


                    ui.add_space(20.0);

                    for network in data.rpc.iter_mut() {
                        ui.horizontal(|ui| {
                            ui.add_space(60.0);
                            ui.add(icons.chain_icon(&network.chain_id));
                            ui.add_space(3.0);
                            let text = RichText::new(network.chain_name())
                                .family(roboto_regular())
                                .size(15.0)
                                .color(Color32::WHITE);
                            ui.label(text);
                        });

                        ui.add_space(10.0);
                        let text_edit = TextEdit::singleline(&mut network.url)
                            .font(font.clone())
                            .text_color(Color32::WHITE)
                            .desired_width(200.0);
                        ui.add(text_edit);
                        ui.add_space(10.0);
                    }

                    if ui.add(save_button).clicked() {
                        match data.save_rpc() {
                            Ok(_) => {
                                trace!("Saved RPC Endpoints");
                                let mut state = SHARED_UI_STATE.write().unwrap();
                                state.network_settings = false;
                            }
                            Err(e) => {
                                error!("Error saving RPC Endpoints: {:?}", e);
                                let mut state = SHARED_UI_STATE.write().unwrap();
                                state.err_msg = ErrorMsg::new(true, e);
                                state.network_settings = false;
                            }
                        }
                    }
                });
            });
    }