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
    pub add: TextureHandle,
    pub view_key: TextureHandle,
    pub tx_settings: TextureHandle,
    pub online: TextureHandle,
    pub offline: TextureHandle,

    // Chain icons
    pub eth: TextureHandle,
    pub bsc: TextureHandle,
    pub base: TextureHandle,
    pub arbitrum: TextureHandle,

    // Currency Icons
    pub eth_coin: TextureHandle,
    pub bnb_coin: TextureHandle,

    // Settings Icons
    pub network: TextureHandle,
    pub wallet: TextureHandle,
    pub rename: TextureHandle,
}

impl IconTextures {
    pub fn new(ctx: &Context) -> Result<Self, anyhow::Error> {
        let copy = load_image_from_memory(include_bytes!("settings/wallet/copy.png"), 16, 16)?;
        let add = load_image_from_memory(include_bytes!("settings/wallet/add.png"), 16, 16)?;
        let tx_settings_icon = load_image_from_memory(include_bytes!("misc/tx_settings.png"), 24, 24)?;
        let online_icon = load_image_from_memory(include_bytes!("misc/online.png"), 24, 24)?;
        let offline_icon = load_image_from_memory(include_bytes!("misc/offline.png"), 24, 24)?;

        // Chain icons
        let eth_icon = load_image_from_memory(include_bytes!("chain/ethereum.png"), 24, 24)?;
        let bsc_icon = load_image_from_memory(include_bytes!("chain/bsc.png"), 24, 24)?;
        let base_icon = load_image_from_memory(include_bytes!("chain/base.png"), 24, 24)?;
        let arbitrum_icon = load_image_from_memory(include_bytes!("chain/arbitrum.png"), 24, 24)?;

        // Currency icons
        let eth_coin = load_image_from_memory(include_bytes!("currency/ethereum.png"), 24, 24)?;
        let bnb_coin = load_image_from_memory(include_bytes!("currency/bnb.png"), 24, 24)?;

        // Settings icons
        let network = load_image_from_memory(include_bytes!("settings/network.png"), 36, 36)?;
        let wallet = load_image_from_memory(include_bytes!("settings/wallet/wallet.png"), 36, 36)?;
        let view_key = load_image_from_memory(include_bytes!("settings/wallet/key.png"), 16, 16)?;
        let rename = load_image_from_memory(include_bytes!("settings/wallet/rename.png"), 16, 16)?;


        let texture_options = TextureOptions::default();

        Ok(Self {
            copy: ctx.load_texture("copy_icon", copy, texture_options),
            add: ctx.load_texture("add_icon", add, texture_options),
            view_key: ctx.load_texture("view_key_icon", view_key, texture_options),
            tx_settings: ctx.load_texture("tx_settings_icon", tx_settings_icon, texture_options),
            online: ctx.load_texture("online", online_icon, texture_options),
            offline: ctx.load_texture("offline", offline_icon, texture_options),
            eth: ctx.load_texture("eth", eth_icon, texture_options),
            bsc: ctx.load_texture("bsc", bsc_icon, texture_options),
            base: ctx.load_texture("base", base_icon, texture_options),
            arbitrum: ctx.load_texture("arbitrum", arbitrum_icon, texture_options),
            eth_coin: ctx.load_texture("eth_coin", eth_coin, texture_options),
            bnb_coin: ctx.load_texture("bnb_coin", bnb_coin, texture_options),
            network: ctx.load_texture("network", network, texture_options),
            wallet: ctx.load_texture("wallet", wallet, texture_options),
            rename: ctx.load_texture("rename", rename, texture_options),
        })
    }

    /// Return Network icon
    pub fn network_icon(&self) -> Image<'static> {
        Image::new(&self.network)
    }

    /// Return Wallet icon
    pub fn wallet_icon(&self) -> Image<'static> {
        Image::new(&self.wallet)
    }


    /// Return the view key icon as [ImageButton]
    pub fn view_key_btn(&self) -> ImageButton {
        ImageButton::new(&self.view_key).rounding(10.0)
    }

    /// Return the export key as [Image]
    pub fn view_key(&self) -> Image<'static> {
        Image::new(&self.view_key)
    }

    /// Return the copy icon as [ImageButton]
    pub fn copy_btn(&self) -> ImageButton {
        ImageButton::new(&self.copy).rounding(10.0)
    }

    /// Return the copy icon as [Image]
    pub fn copy(&self) -> Image<'static> {
        Image::new(&self.copy)
    }

    /// Return the add icon as [ImageButton]
    pub fn add_btn(&self) -> ImageButton {
        ImageButton::new(&self.add).rounding(10.0)
    }

    /// Return the add icon as [Image]
    pub fn add(&self) -> Image<'static> {
        Image::new(&self.add)
    }

    /// Return the rename icon as [Image]
    pub fn rename(&self) -> Image<'static> {
        Image::new(&self.rename)
    }


    /// Return the chain icon based on the chain_id
    pub fn chain_icon(&self, id: &u64) -> Image<'static> {
        match id {
            1 => Image::new(&self.eth),
            56 => Image::new(&self.bsc),
            8453 => Image::new(&self.base),
            42161 => Image::new(&self.arbitrum),
            _ => Image::new(&self.eth),
        }
    }

    /// Return the native currency icon based on the chain_id
    pub fn currency_icon(&self, id: u64) -> Image<'static> {
        match id {
            1 => Image::new(&self.eth_coin),
            56 => Image::new(&self.bnb_coin),
            8453 => Image::new(&self.eth_coin),
            42161 => Image::new(&self.eth_coin),
            _ => Image::new(&self.eth_coin),
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