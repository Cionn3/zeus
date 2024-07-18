use eframe::egui::{self, SelectableLabel};
use egui::{
    vec2, Align, Align2, Button, Color32, FontId, Layout, Response, RichText, TextEdit, Ui,
};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tracing::trace;

use crossbeam::channel::Sender;

use crate::{fonts::roboto_regular, icons::IconTextures};

use zeus_backend::types::Request;
use zeus_chain::{alloy::primitives::Address, defi_types::currency::Currency, utils::format_wei};
use zeus_shared_types::{
    AppData, ErrorMsg, SelectedCurrency, UiState, SwapUIState, SHARED_UI_STATE, SWAP_UI_STATE,
};


/// Manages the state of the swap UI
pub struct SwapUI {
    /// Send Request to the backend
    pub front_sender: Option<Sender<Request>>,

    pub open: bool,

    pub state: Arc<RwLock<SwapUIState>>,
}

impl Default for SwapUI {
    fn default() -> Self {
        Self {
            front_sender: None,
            open: true,
            state: SWAP_UI_STATE.clone(),
        }
    }
}