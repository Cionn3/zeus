use eframe::{
    egui::{
        style::{Selection, WidgetVisuals, Widgets},
        Color32, Frame, Rounding, Stroke, Visuals,
        Ui, RichText
    },
    epaint::{Margin, Shadow, vec2},
};
use egui::{FontId, TextEdit};
use crate::{fonts::roboto_regular, AppData};
use zeus_backend::types::Request;
use crate::gui::{swap_ui::SwapUI, settings::SettingsUi};
use lazy_static::lazy_static;
use crossbeam::channel::Sender;

pub mod swap_ui;
pub mod settings;
pub mod misc;

lazy_static! {
    pub static ref THEME: ZeusTheme = ZeusTheme::default();
}


/// The Graphical User Interface for [crate::ZeusApp]
pub struct GUI {
    /// Send data to backend
    pub sender: Option<Sender<Request>>,

    pub swap_ui: SwapUI,

    pub settings: SettingsUi,

    /// Err Message popup
    /// 
    /// (on/off, Error)
    pub err_msg: (bool, anyhow::Error),

    /// Info Message
    /// 
    /// (on/off, String)
    pub info_msg: (bool, String),
}

impl Default for GUI {
    fn default() -> Self {
        Self {
            sender: None,
            swap_ui: SwapUI::default(),
            settings: SettingsUi::default(),
            err_msg: (false, anyhow::Error::msg("")),
            info_msg: (false, "".to_string()),
        }
    }
}

impl GUI {
    /// Render and handle the menu
    pub fn menu(&mut self, ui: &mut Ui) {
       let swap = RichText::new("Swap").family(roboto_regular()).size(20.0);
       let settings = RichText::new("Settings").family(roboto_regular()).size(20.0);
       let networks = RichText::new("Networks").family(roboto_regular()).size(15.0);
       let test_err = RichText::new("Test Error").family(roboto_regular()).size(15.0);
       let test_info = RichText::new("Test Info").family(roboto_regular()).size(15.0);


       ui.vertical(|ui| {

        if ui.label(test_err).clicked() {
            self.err_msg = (true, anyhow::Error::msg("Test Error"));
        }

        if ui.label(test_info).clicked() {
            self.info_msg = (true, "Test Info".to_string());
        }

        if ui.label(swap).clicked() {
            self.settings.networks_on = false;
            self.swap_ui.on = true;
        }

        ui.add_space(10.0);

        ui.collapsing(settings, |ui| {
            
            if ui.label(networks).clicked() {
                self.swap_ui.on = false;
                self.settings.networks_on = true;
            }
        });
        
    });
    }

    /// Render Network Settings UI
    pub fn networks_ui(&mut self, ui: &mut Ui, data: &mut AppData) {

        if !self.settings.networks_on {
           return;
        }
        
        let description = RichText::new("RPC Endpoints, Currently only supports Websockets").family(roboto_regular()).size(20.0);
        let font = FontId::new(15.0, roboto_regular());

        let frame = Frame {
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
        };

        frame.show(ui, |ui| {
            ui.set_max_size(egui::vec2(400.0, 500.0));

            ui.vertical_centered(|ui| {
                ui.label(description);
                ui.add_space(30.0);

                
                ui.vertical_centered(|ui| {
                   
                    for network in data.rpc.iter_mut() {
                        let label = RichText::new(network.chain_name()).family(roboto_regular()).size(15.0).color(THEME.colors.white);
                        ui.label(label);

                        ui.add_space(10.0);

                        let text_field = TextEdit::singleline(&mut network.url).font(font.clone()).text_color(THEME.colors.dark_gray);
                        ui.add(text_field);
                        ui.add_space(10.0);
                    }
                });
                ui.horizontal_centered(|ui| {

                    // !DEBUG
                    let print_rpc = RichText::new("Print Rpc").family(roboto_regular()).size(15.0).color(THEME.colors.white);
                    let current_chainid = RichText::new("Current ChainId").family(roboto_regular()).size(15.0).color(THEME.colors.white);
                    
                    ui.add_space(50.0);

                    if ui.button(print_rpc).clicked() {
                        println!("RPC: {:?}", data.rpc);
                    }


                    if ui.button(current_chainid).clicked() {
                        println!("Current ChainId: {:?}", data.chain_id.name());
                    }
                });
            });
        });

       

       
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