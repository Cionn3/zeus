use std::sync::Arc;

use eframe::{ egui, CreationContext };
use egui::{
    vec2, Align2, ComboBox, Context, Style, TextEdit, Ui
};

use crossbeam::channel::{ bounded, Sender, Receiver };


use crate::{
    fonts::get_fonts,
    gui::{ ZeusTheme, GUI, misc::{ login_screen, new_profile_screen, rich_text, frame } },
};

use zeus_backend::{ Backend, types::{ Request, Response } };
use zeus_types::app_data::AppData;

pub mod gui;
pub mod fonts;

/// The main application struct
pub struct ZeusApp {
    /// The GUI components of the application
    pub gui: GUI,

    /// Send Data to backend
    pub front_sender: Option<Sender<Request>>,

    /// Receive Data from backend
    pub back_receiver: Option<Receiver<Response>>,

    /// The app data of the application
    pub data: AppData,
}

impl Default for ZeusApp {
    fn default() -> Self {
        Self {
            gui: GUI::default(),
            front_sender: None,
            back_receiver: None,
            data: AppData::default(),
        }
    }
}

impl ZeusApp {
    pub fn new(cc: &CreationContext) -> Self {
        let mut app = Self::default();
        app.config_style(&cc.egui_ctx);

        match app.data.load_rpc() {
            Ok(_) => {}
            Err(e) => {
                println!("Error Loading rpc.json: {}", e);
            }
        }

        let (front_sender, front_receiver) = bounded(1);
        let (back_sender, back_receiver) = bounded(1);

        app.gui.swap_ui.front_sender = Some(front_sender.clone());
        app.gui.sender = Some(front_sender.clone());

        std::thread::spawn(move || {
            Backend::new(back_sender, front_receiver).init();
        });

        app.front_sender = Some(front_sender);
        app.back_receiver = Some(back_receiver);
        app
    }

    fn config_style(&self, ctx: &Context) {
        let style = Style {
            visuals: ZeusTheme::default().visuals,
            ..Style::default()
        };
        ctx.set_fonts(get_fonts());
        ctx.set_style(style);
    }

    /// Send a request to backend
    fn send_request(&self, request: Request) {
        if let Some(sender) = &self.front_sender {
            sender.send(request).unwrap();
        }
    }

    fn draw_login(&mut self, ui: &mut Ui) {
        if self.data.profile_exists && !self.data.logged_in {
            login_screen(ui, self);
        }

        if self.data.new_profile_screen {
            new_profile_screen(ui, self);
        }
    }

    // TODO: show chain icon
    fn select_chain(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ComboBox::from_label("")
                .selected_text(self.data.chain_id.name())
                .show_ui(ui, |ui| {
                    for id in self.data.networks.iter().map(|(chain_id, _)| chain_id.clone()) {
                       if ui.selectable_value(&mut self.data.chain_id, id.clone(), id.name()).clicked() {
                            println!("Selected Chain: {:?}", id);
                            self.send_request(Request::GetClient { chain_id: id, rpcs: self.data.rpc.clone() });
                        }
                    }   
                });
        });
    }

    fn wallet_selection(&mut self, ui: &mut Ui) {
         
        if !self.data.logged_in || self.data.new_profile_screen {
            return;
        }

        ui.vertical(|ui| {
            ui.horizontal(|ui| {

                frame().show(ui, |ui| {
                    let current_wallet = self.data.profile.current_wallet_name();

                    // TODO: an oracle to fetch the balance at specific intervals
                    let balance = rich_text(&self.data.native_balance(), 15.0);

                    let coin = rich_text(&self.data.native_coin(), 15.0);

                    ComboBox::from_label("")
                        .selected_text(current_wallet)
                        .show_ui(ui, |ui| {
                            for wallet in &self.data.profile.wallets {
                                ui.selectable_value(
                                    &mut self.data.profile.current_wallet,
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
                    self.gui.settings.wallet_popup = (true, "New");
                }

                ui.add_space(5.0);

                if ui.button("Import Wallet").clicked() {
                    self.gui.settings.wallet_popup = (true, "Import Wallet");
                }
            });

            ui.vertical(|ui| {
                if ui.button("Copy Address").clicked() {

                    let curr_wallet = self.data.profile.current_wallet.clone();

                    let curr_wallet = match curr_wallet {
                        Some(wallet) => wallet,
                        None => {
                            self.gui.err_msg = (true, anyhow::Error::msg("No Wallet Selected"));
                            return;
                        }
                    };
                    // TODO copy address to clipboard
                }

                ui.add_space(5.0);

                if ui.button("Export Private Key").clicked() {
                    // TODO: export private key
                }
            });


        });
            ui.add_space(10.0);

        });
    }

    fn wallet_popup(&mut self, ui: &mut Ui) {

        if !self.gui.settings.wallet_popup.0 {
            return;
        }

        egui::Window::new(self.gui.settings.wallet_popup.1)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .collapsible(false)
        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {

           if self.gui.settings.wallet_popup.1 == "Import Wallet" {

            let label = rich_text("Private key:", 15.0);
            let name_label = rich_text("Wallet Name (Optional):", 15.0);

            let key_field = TextEdit::singleline(&mut self.data.private_key).desired_width(200.0).password(true);
            let name_field = TextEdit::singleline(&mut self.data.wallet_name).desired_width(200.0);

            
                ui.label(label);
                ui.add_space(5.0);
                ui.add(key_field);
                ui.add_space(5.0);
                ui.label(name_label);
                ui.add_space(5.0);
                ui.add(name_field);
                ui.add_space(5.0);

               if ui.button("Import").clicked() {

                    match self.data.profile.import_wallet(self.data.wallet_name.clone(), self.data.private_key.clone()) {
                        Ok(_) => {}
                        Err(e) => {
                            self.gui.err_msg = (true, e);
                        }
                    }

                    self.gui.settings.wallet_popup = (false, "Import Wallet");
                    self.data.private_key = "".to_string();
                    self.data.wallet_name = "".to_string();
                    self.send_request(Request::SaveProfile { profile: self.data.profile.clone() });
                    
                }
               
            } else if self.gui.settings.wallet_popup.1 == "New" {
                let label = rich_text("Wallet Name (Optional):", 15.0);
                let name_field = TextEdit::singleline(&mut self.data.wallet_name).desired_width(200.0);

                ui.label(label);
                ui.add_space(5.0);
                ui.add(name_field);
                ui.add_space(5.0);

                if ui.button("Create").clicked() {
                    self.data.profile.new_wallet(self.data.wallet_name.clone());
                    self.gui.settings.wallet_popup = (false, "New");
                    self.data.wallet_name = "".to_string();
                    self.send_request(Request::SaveProfile { profile: self.data.profile.clone() });
                }
            }
            });
        });
        
    }

    /// Show an error message if needed
    fn err_msg(&mut self, ui: &mut Ui) {

        if !self.gui.err_msg.0 {
            return;
        }

        egui::Window::new("Error")
        .resizable(false)
        .anchor(Align2::CENTER_TOP, vec2(0.0, 0.0))
        .collapsible(false)
        .title_bar(false)
        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                let msg = self.gui.err_msg.1.to_string();
                let msg_text = rich_text(&msg, 16.0);
                let close_text = rich_text("Close", 16.0);

                ui.label(msg_text);
                ui.add_space(5.0);
                if ui.button(close_text).clicked() {
                    self.gui.err_msg.0 = false;
                }
            });
        });
}

    /// Show an info message if needed
    fn info_msg(&mut self, ui: &mut Ui) {

        if !self.gui.info_msg.0 {
            return;
        }

        ui.vertical_centered_justified(|ui| {
        frame()
        .show(ui, |ui| {
            ui.set_max_size(vec2(100.0, 50.0));
           
                let msg = self.gui.info_msg.1.clone();
                let msg_text = rich_text(&msg, 16.0);
                let close_text = rich_text("Close", 16.0);

                ui.label(msg_text);
                ui.add_space(5.0);
                if ui.button(close_text).clicked() {
                    self.gui.info_msg.0 = false;
                }  
        });
    });
}

}

// Main Event Loop Of The Window
// This is where we draw the UI
impl eframe::App for ZeusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        //TODO: avoid unwrap
        if let Some(receive) = &self.back_receiver {
            match receive.try_recv() {
                Ok(response) => {
                    match response {
                        Response::SimSwap { result } => {
                            println!("Swap Response: {:?}", result);
                        }

                        Response::Balance(balance) => {
                            println!("Balance: {}", balance);
                        }

                        Response::SaveProfile(res) => {
                            if res.is_err() {
                                self.gui.err_msg = (true, res.unwrap_err());
                            }
                        }

                        Response::GetClient(res) => {
                            if res.is_err() {
                                self.gui.err_msg = (true, res.unwrap_err());
                            } else {
                                self.data.ws_client = Some(Arc::new(res.unwrap()));
                            }
                        }
                    }
                }
                Err(_) => {}
            }
        }

        // Draw the UI that belongs to the Top Panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                self.wallet_selection(ui);
                self.wallet_popup(ui);
                self.info_msg(ui);
            });
        });

        // Draw the UI that belongs to the Central Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                self.err_msg(ui);
                self.draw_login(ui);
            });

            if !self.data.logged_in || self.data.new_profile_screen {
                return;
            }

            ui.vertical_centered_justified(|ui| {
                ui.add_space(100.0);
                self.gui.swap_ui.swap_panel(ui);
                self.gui.networks_ui(ui, &mut self.data);
            });
        });

        // Draw the UI that belongs to the Left Panel
        egui::SidePanel
            ::left("left_panel")
            .exact_width(170.0)
            .show(ctx, |ui| {
                self.select_chain(ui);

                ui.add_space(10.0);

                self.gui.menu(ui);
            });
    }
}
