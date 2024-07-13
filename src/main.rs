// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

use zeus_gui::app::{ZeusApp, WIDTH, HEIGHT};


fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([WIDTH, HEIGHT]),
        ..Default::default()
    };

    eframe::run_native(
        "Zeus",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(ZeusApp::new(cc)))
        })
    )
}
