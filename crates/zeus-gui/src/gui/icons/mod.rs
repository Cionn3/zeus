use egui::{ include_image, Image, Sense };

/// Get the chain icon from the given chain_id
pub fn get_chain_icon(id: u64) -> Image<'static> {
    match id {
        1 => Image::new(include_image!("eth.png")).max_width(32.0).rounding(10.0),
        56 => Image::new(include_image!("bsc.svg")).max_width(32.0).rounding(10.0),
        8453 => Image::new(include_image!("base.png")).max_width(32.0).rounding(10.0),
        42161 => Image::new(include_image!("arbitrum.png")).max_width(32.0).rounding(10.0),
        _ => Image::new(include_image!("eth.png")).max_width(32.0).rounding(10.0),
    }
}

pub fn tx_settings_icon() -> Image<'static> {
    Image::new(include_image!("tx_settings_icon.svg"))
        .max_width(24.0)
        .rounding(20.0)
        .sense(Sense::click())
        .bg_fill(egui::Color32::WHITE)
}

pub fn green_circle() -> Image<'static> {
    Image::new(include_image!("green-circle.png"))
        .max_width(24.0)
        .rounding(20.0)
}

pub fn red_circle() -> Image<'static> {
    Image::new(include_image!("red-circle.png"))
        .max_width(24.0)
        .rounding(20.0)
}

/// Return a green or red circle icon based on if we are connected or not to a node
pub fn connected_icon(connected: bool) -> Image<'static> {
    match connected {
        true => green_circle(),
        false => red_circle(),
    }
}