use eframe::{
    egui::{
        style::{Selection, WidgetVisuals, Widgets}, Color32, Frame, RichText, Rounding, Stroke, Ui, Visuals,
        Align2, ComboBox
    },
    epaint::{vec2, Margin},
};
use egui::{FontId, TextEdit};
use misc::{rich_text, frame};
use crate::{fonts::roboto_regular, AppData};

use zeus_backend::types::Request;
use crate::gui::{swap_ui::SwapUI, state::*};
use lazy_static::lazy_static;
use crossbeam::channel::Sender;

pub mod swap_ui;
pub mod icons;
pub mod state;
pub mod misc;

lazy_static! {
    pub static ref THEME: ZeusTheme = ZeusTheme::default();
}



/// The Graphical User Interface for [crate::ZeusApp]
pub struct GUI {
    /// Send data to backend
    pub sender: Option<Sender<Request>>,

    pub swap_ui: SwapUI,

}

impl Default for GUI {
    fn default() -> Self {
        Self {
            sender: None,
            swap_ui: SwapUI::default(),
        }
    }
}

impl GUI {

    /// Send a request to the backend
    pub fn send_request(&mut self, request: Request, shared_state: &mut SharedUiState) {
        if let Some(sender) = &self.sender {
            match sender.send(request) {
                Ok(_) => {}
                Err(e) => {
                   shared_state.err_msg = ErrorMsg::new(true, e);
                }
            }
        }
    }

    /// Render and handle the menu
    pub fn menu(&mut self, ui: &mut Ui, shared_state: &mut SharedUiState) {
       let swap = RichText::new("Swap").family(roboto_regular()).size(20.0);
       let settings = RichText::new("Settings").family(roboto_regular()).size(20.0);
       let networks = RichText::new("Networks").family(roboto_regular()).size(15.0);
       let test_err = RichText::new("Test Error").family(roboto_regular()).size(15.0);
       let test_info = RichText::new("Test Info").family(roboto_regular()).size(15.0);


       ui.vertical(|ui| {

        if ui.label(test_err).clicked() {
           shared_state.err_msg = ErrorMsg::new(true, "Test Error");
        }

        if ui.label(test_info).clicked() {
            shared_state.info_msg = InfoMsg::new(true, "Test Info");
        }

        if ui.label(swap).clicked() {
            shared_state.networks_on = false;
            self.swap_ui.on = true;
        }

        ui.add_space(10.0);

        ui.collapsing(settings, |ui| {
            
            if ui.label(networks).clicked() {
                self.swap_ui.on = false;
                shared_state.networks_on = true;
            }
        });
        
    });
    }

    /// Render Network Settings UI
    pub fn networks_ui(&mut self, ui: &mut Ui, data: &mut AppData, shared_state: &mut SharedUiState) {

        if !shared_state.networks_on {
           return;
        }
        
        let description = RichText::new("RPC Endpoints, Currently only supports Websockets").family(roboto_regular()).size(20.0);
        let font = FontId::new(15.0, roboto_regular());


        frame().show(ui, |ui| {
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

    /// Render the UI repsonsible for managing the wallets
    pub fn wallet_ui (&mut self, ui: &mut Ui, data: &mut AppData, shared_state: &mut SharedUiState) {
        self.wallet_selection(ui, data, shared_state);
        self.create_or_import_wallet(ui, data, shared_state);
        self.export_key_ui(ui, data, shared_state);
        self.show_exported_key(ui, shared_state);

    }

    fn create_or_import_wallet(&mut self, ui: &mut Ui, data: &mut AppData, shared_state: &mut SharedUiState) {

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
                    self.send_request(Request::SaveProfile { profile: data.profile.clone() }, shared_state);
                    
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
                    self.send_request(Request::SaveProfile { profile: data.profile.clone() }, shared_state);
                }
                if ui.button("Close").clicked() {
                    shared_state.wallet_popup = (false, "New");
                }
            }
            });
        });
        
    }

    fn wallet_selection(&mut self, ui: &mut Ui, data: &mut AppData, shared_state: &mut SharedUiState) {
         
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

    pub fn export_key_ui(&mut self, ui: &mut Ui, data: &mut AppData, shared_state: &mut SharedUiState) {
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

   pub fn show_exported_key(&mut self, ui: &mut Ui, shared_state: &mut SharedUiState) {
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