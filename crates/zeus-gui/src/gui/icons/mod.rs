use eframe::egui::{
    Context,
    epaint::textures::TextureOptions,
    Image,
    ColorImage,
    TextureHandle,
    ImageButton,
};

use image::imageops::FilterType;


/// A collection of icons used in the GUI
#[derive(Clone)]
pub struct IconTextures {
    pub copy: TextureHandle,
    pub wallet_new: TextureHandle,
    pub export_key: TextureHandle,
    pub tx_settings: TextureHandle,
    pub online: TextureHandle,
    pub offline: TextureHandle,

    // Chain icons
    pub eth: TextureHandle,
    pub bsc: TextureHandle,
    pub base: TextureHandle,
    pub arbitrum: TextureHandle,
}

impl IconTextures {
    pub fn new(ctx: &Context) -> Result<Self, anyhow::Error> {
        let copy_icon = load_image_from_memory(include_bytes!("wallet/copy.png"), 24, 24)?;
        let wallet_new_icon = load_image_from_memory(include_bytes!("wallet/new.png"), 24, 24)?;
        let export_key_icon = load_image_from_memory(include_bytes!("wallet/export.png"), 24, 24)?;
        let tx_settings_icon = load_image_from_memory(include_bytes!("misc/tx_settings.png"), 24, 24)?;
        let online_icon = load_image_from_memory(include_bytes!("misc/online.png"), 24, 24)?;
        let offline_icon = load_image_from_memory(include_bytes!("misc/offline.png"), 24, 24)?;

        // Chain icons
        let eth_icon = load_image_from_memory(include_bytes!("chain/eth.png"), 24, 24)?;
        let bsc_icon = load_image_from_memory(include_bytes!("chain/bsc.png"), 24, 24)?;
        let base_icon = load_image_from_memory(include_bytes!("chain/base.png"), 24, 24)?;
        let arbitrum_icon = load_image_from_memory(include_bytes!("chain/arbitrum.png"), 24, 24)?;

        let texture_options = TextureOptions::default();

        Ok(Self {
            copy: ctx.load_texture("copy_icon", copy_icon, texture_options),
            wallet_new: ctx.load_texture("wallet_new_icon", wallet_new_icon, texture_options),
            export_key: ctx.load_texture("export_key_icon", export_key_icon, texture_options),
            tx_settings: ctx.load_texture("tx_settings_icon", tx_settings_icon, texture_options),
            online: ctx.load_texture("online", online_icon, texture_options),
            offline: ctx.load_texture("offline", offline_icon, texture_options),
            eth: ctx.load_texture("eth", eth_icon, texture_options),
            bsc: ctx.load_texture("bsc", bsc_icon, texture_options),
            base: ctx.load_texture("base", base_icon, texture_options),
            arbitrum: ctx.load_texture("arbitrum", arbitrum_icon, texture_options),
        })
    }

    /// Return the export key icon
    pub fn export_key_icon(&self) -> ImageButton {
        ImageButton::new(&self.export_key).rounding(10.0)
    }

    /// Return the copy icon as [ImageButton]
    pub fn copy_icon(&self) -> ImageButton {
        ImageButton::new(&self.copy).rounding(10.0)
    }

    /// Return the wallet new icon as [ImageButton]
    pub fn wallet_new_icon(&self) -> ImageButton {
        ImageButton::new(&self.wallet_new).rounding(10.0)
    }

    /// Return the chain icon based on the chain_id
    pub fn chain_icon(&self, id: u64) -> Image<'static> {
        match id {
            1 => Image::new(&self.eth),
            56 => Image::new(&self.bsc),
            8453 => Image::new(&self.base),
            42161 => Image::new(&self.arbitrum),
            _ => Image::new(&self.eth),
        }
    }

    /// Return the tx settings icon as [ImageButton]
    pub fn tx_settings_icon(&self) -> ImageButton {
        ImageButton::new(&self.tx_settings).rounding(10.0)
    }

    /// Return the online icon
    pub fn online_icon(&self) -> Image<'static> {
        Image::new(&self.online)
    }

    /// Return the offline icon
    pub fn offline_icon(&self) -> Image<'static> {
        Image::new(&self.offline)
    }

    /// Return an online or offline icon based on the connected status
    pub fn connected_icon(&self, connected: bool) -> Image<'static> {
        match connected {
            true => self.online_icon(),
            false => self.offline_icon(),
        }
    }
}

fn load_image_from_memory(image_data: &[u8], width: u32, height: u32) -> Result<ColorImage, image::ImageError> {
    let image = image::load_from_memory(image_data)?;
    let resized_image = image.resize(width, height, FilterType::Lanczos3);
    let size = [resized_image.width() as _, resized_image.height() as _];
    let image_buffer = resized_image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}