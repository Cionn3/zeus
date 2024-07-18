use zeus_core::lazy_static::lazy_static;
use std::sync::{Arc, RwLock};
use super::{error::ErrorMsg, info::InfoMsg};

lazy_static! {

    /// See [SharedUiState]
    /// 
    /// This can be safely shared across all tasks
    pub static ref SHARED_UI_STATE: Arc<RwLock<SharedUiState>> = Arc::new(
        RwLock::new(SharedUiState::default())
    );
}



/// Shared State for some GUI components
/// 
/// We use this to turn on/off some UI components that are not part of the main UI
/// 
/// For convenience we use a thread-safe [SHARED_UI_STATE] instance to manage the state
/// 
/// For example an [ErrorMsg] can be set here using 
/// ```
/// let mut state = SHARED_UI_STATE.write().unwrap();
/// state.err_msg.show(err);
/// ```
#[derive(Clone)]
pub struct SharedUiState {

    /// Search Query for a token
    pub search_query: String,

    /// New/Import Wallet UI on/off
    ///
    /// (on/off, "New"/"Import Wallet")
    pub wallet_popup: (bool, &'static str),

    /// New Wallet Window
    pub new_wallet_window_on: bool,

    /// Generate Wallet UI
    pub generate_wallet_on: bool,

    /// Import Wallet UI
    pub import_wallet_window_on: bool,

    /// Export wallet Key UI on/off
    pub export_key_ui: bool,

    /// Exported key window on/off
    pub exported_key_window: (bool, String),

    /// Network Settings
    pub network_settings: bool,

    /// TxSettings popup on/off
    pub tx_settings_on: bool,

    /// Error message to show in the UI
    pub err_msg: ErrorMsg,

    /// Info message to show in the UI
    pub info_msg: InfoMsg,
}

impl Default for SharedUiState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            wallet_popup: (false, "New"),
            new_wallet_window_on: false,
            generate_wallet_on: false,
            import_wallet_window_on: false,
            network_settings: false,
            export_key_ui: false,
            exported_key_window: (false, String::new()),
            tx_settings_on: false,
            err_msg: ErrorMsg::default(),
            info_msg: InfoMsg::default(),
        }
    }
}