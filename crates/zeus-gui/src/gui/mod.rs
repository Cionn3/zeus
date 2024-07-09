use eframe::{
    egui::{
        style::{ Selection, WidgetVisuals, Widgets },
        vec2,
        Color32,
        FontId,
        Frame,
        RichText,
        Rounding,
        Stroke,
        TextEdit,
        Ui,
        Visuals,
    },
    epaint::Margin,
};

use std::sync::Arc;
use misc::frame;
use crate::{
    fonts::roboto_regular,
    gui::{ icons::IconTextures, swap_ui::SwapUI },
};

use wallet_ui::wallet_ui;

use zeus_backend::types::Request;
use zeus_shared_types::{ SHARED_UI_STATE, ErrorMsg, AppData };

use lazy_static::lazy_static;
use crossbeam::channel::Sender;

pub mod swap_ui;
pub mod wallet_ui;
pub mod icons;
pub mod misc;

lazy_static! {
    pub static ref THEME: ZeusTheme = ZeusTheme::default();
}

/// The Graphical User Interface for [crate::ZeusApp]
pub struct GUI {
    /// Send data to backend
    pub sender: Option<Sender<Request>>,

    pub swap_ui: SwapUI,

    pub icons: Arc<IconTextures>,
}

impl GUI {
    pub fn new_default(icons: Arc<IconTextures>) -> Self {
        Self {
            sender: None,
            swap_ui: SwapUI::default(),
            icons,
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

    /// Render and handle the menu
    pub fn menu(&mut self, ui: &mut Ui, data: &mut AppData) {
        let swap = RichText::new("Swap").family(roboto_regular()).size(20.0);
        let settings = RichText::new("Settings").family(roboto_regular()).size(20.0);
        let networks = RichText::new("Networks").family(roboto_regular()).size(15.0);
        let base_fee = RichText::new("Base Fee").family(roboto_regular()).size(15.0);

        ui.vertical(|ui| {
            ui.label(base_fee);
            ui.label(
                RichText::new(&data.block_info.1.readable()).family(roboto_regular()).size(15.0)
            );
            ui.add_space(10.0);

            if ui.label(swap).clicked() {
                let mut state = SHARED_UI_STATE.write().unwrap();
                state.networks_on = false;
                state.swap_ui_on = true;
            }

            ui.add_space(10.0);

            ui.collapsing(settings, |ui| {
                if ui.label(networks).clicked() {
                    let mut state = SHARED_UI_STATE.write().unwrap();
                    state.swap_ui_on = false;
                    state.networks_on = true;
                }
            });
        });
    }

    /// Render Network Settings UI
    pub fn networks_ui(&mut self, ui: &mut Ui, data: &mut AppData) {
        {
            let state = SHARED_UI_STATE.read().unwrap();
            if !state.networks_on {
                return;
            }
        }

        let description = RichText::new("RPC Endpoints, Currently only supports Websockets")
            .family(roboto_regular())
            .size(20.0);
        let font = FontId::new(15.0, roboto_regular());

        frame().show(ui, |ui| {
            ui.set_max_size(vec2(400.0, 500.0));

            ui.vertical_centered(|ui| {
                ui.label(description);
                ui.add_space(30.0);

                ui.vertical_centered(|ui| {
                    for network in data.rpc.iter_mut() {
                        let label = RichText::new(network.chain_name())
                            .family(roboto_regular())
                            .size(15.0)
                            .color(THEME.colors.white);
                        ui.label(label);

                        ui.add_space(10.0);

                        let text_field = TextEdit::singleline(&mut network.url)
                            .font(font.clone())
                            .text_color(THEME.colors.dark_gray);
                        ui.add(text_field);
                        ui.add_space(10.0);
                    }
                });
                ui.horizontal_centered(|ui| {
                    let save = RichText::new("Save")
                        .family(roboto_regular())
                        .size(15.0)
                        .color(THEME.colors.white);

                    ui.add_space(50.0);

                    if ui.button(save).clicked() {
                        match data.save_rpc() {
                            Ok(_) => {}
                            Err(e) => {
                                let mut state = SHARED_UI_STATE.write().unwrap();
                                state.err_msg = ErrorMsg::new(true, e);
                            }
                        }
                    }
                });
            });
        });
    }

    /// Render the UI repsonsible for managing the wallets
    pub fn render_wallet_ui(&mut self, ui: &mut Ui, data: &mut AppData) {
        wallet_ui(ui, data, &self);
    }
}

// credits: https://github.com/4JX/mCubed/blob/master/main/src/ui/app_theme.rs
/// Holds the Theme Settings for the Main App
pub struct ZeusTheme {
    pub colors: Colors,
    pub visuals: Visuals,
    pub rounding: RoundingTypes,
    pub default_panel_frame: Frame,
    pub prompt_frame: Frame,
}

impl Default for ZeusTheme {
    fn default() -> Self {
        let colors = Colors::default();

        let widgets = Widgets {
            noninteractive: WidgetVisuals {
                bg_fill: colors.gray, // window background color
                weak_bg_fill: colors.light_gray,
                bg_stroke: Stroke::new(1.0, colors.dark_gray), // separators, indentation lines, windows outlines
                fg_stroke: Stroke::new(1.0, Color32::from_gray(140)), // normal text color
                rounding: Rounding::same(2.0),
                expansion: 0.0,
            },
            inactive: WidgetVisuals {
                bg_fill: colors.dark_gray, // button background
                weak_bg_fill: colors.darker_gray,
                bg_stroke: Stroke::default(),
                fg_stroke: Stroke::new(1.0, Color32::from_gray(180)), // button text
                rounding: Rounding::same(2.0),
                expansion: 0.0,
            },
            hovered: WidgetVisuals {
                bg_fill: Color32::from_gray(70),
                weak_bg_fill: colors.darker_gray,
                bg_stroke: Stroke::new(1.0, Color32::from_gray(150)), // e.g. hover over window edge or button
                fg_stroke: Stroke::new(1.5, Color32::from_gray(240)),
                rounding: Rounding::same(3.0),
                expansion: 1.0,
            },
            active: WidgetVisuals {
                bg_fill: Color32::from_gray(55),
                weak_bg_fill: colors.silver,
                bg_stroke: Stroke::new(1.0, Color32::WHITE),
                fg_stroke: Stroke::new(2.0, Color32::WHITE),
                rounding: Rounding::same(2.0),
                expansion: 1.0,
            },
            open: WidgetVisuals {
                bg_fill: Color32::from_gray(27),
                weak_bg_fill: colors.darker_gray,
                bg_stroke: Stroke::new(1.0, Color32::from_gray(60)),
                fg_stroke: Stroke::new(1.0, Color32::from_gray(210)),
                rounding: Rounding::same(2.0),
                expansion: 0.0,
            },
        };

        let selection = Selection {
            bg_fill: colors.light_gray,
            ..Selection::default()
        };

        let visuals = Visuals {
            dark_mode: true,
            override_text_color: Some(colors.white),
            widgets,
            selection,
            extreme_bg_color: colors.silver, // affects color of RichText
            ..Visuals::default()
        };

        let default_panel_frame = Frame {
            inner_margin: Margin::same(8.0),
            fill: colors.gray,
            ..Frame::default()
        };

        let rounding = RoundingTypes::default();
        let prompt_frame = default_panel_frame.rounding(rounding.big);

        Self {
            colors,
            visuals,
            default_panel_frame,
            prompt_frame,
            rounding,
        }
    }
}

pub struct RoundingTypes {
    pub small: Rounding,
    pub big: Rounding,
}

impl Default for RoundingTypes {
    fn default() -> Self {
        Self {
            small: Rounding::same(2.0),
            big: Rounding::same(4.0),
        }
    }
}

pub struct Colors {
    pub white: Color32,
    pub silver: Color32,
    pub gray: Color32,
    pub dark_gray: Color32,
    pub darker_gray: Color32,
    pub light_gray: Color32,
    pub lighter_gray: Color32,
    pub error_message: Color32,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            white: Color32::from_rgb(255, 255, 255),
            silver: Color32::from_rgb(192, 192, 192),
            gray: Color32::from_rgb(58, 58, 58),
            dark_gray: Color32::from_rgb(38, 38, 38),
            darker_gray: Color32::from_rgb(22, 22, 22),
            light_gray: Color32::from_rgb(85, 85, 85),
            lighter_gray: Color32::from_rgb(120, 120, 120),
            error_message: Color32::from_rgb(211, 80, 80),
        }
    }
}
