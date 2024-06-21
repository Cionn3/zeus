use lazy_static::lazy_static;
use std::sync::{Arc, RwLock};

lazy_static!{
    pub static ref SHARED_UI_STATE: Arc<RwLock<SharedUiState>> = Arc::new(RwLock::new(SharedUiState::default()));
}

/// Error message to show in the UI
#[derive(Clone, Default)]
pub struct ErrorMsg {
    pub on: bool,
    
    pub msg: String,
}

impl ErrorMsg {
    pub fn new<T>(on: bool, msg: T) -> Self
    where
        T: ToString,
    {
        Self {
            on,
            msg: msg.to_string(),
        }
    }
}


/// Info message to show in the UI
#[derive(Clone, Default)]
pub struct InfoMsg {
    pub on: bool,
    
    pub msg: String,
}

impl InfoMsg {
    pub fn new<T>(on: bool, msg: T) -> Self
    where
        T: ToString,
    {
        Self {
            on,
            msg: msg.to_string(),
        }
    }
}


/// Shared State for some GUI components
#[derive(Clone, Default)]
pub struct SharedUiState {
    /// Network settings UI on/off
    pub networks_on: bool,

    /// New/Import Wallet UI on/off
    ///
    /// (on/off, "New"/"Import Wallet")
    pub wallet_popup: (bool, &'static str),

    /// Export wallet Key UI on/off
    pub export_key_ui: bool,

    /// Exported key window on/off
    pub exported_key_window: (bool, String),

    /// TxSettings popup on/off
    pub tx_settings_on: bool,

    /// Error message to show in the UI
    pub err_msg: ErrorMsg,

    /// Info message to show in the UI
    pub info_msg: InfoMsg,
}