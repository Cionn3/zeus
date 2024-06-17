use egui::{Image, include_image};

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