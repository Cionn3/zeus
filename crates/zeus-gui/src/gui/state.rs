

/// Error message to show in the UI
#[derive(Clone)]
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

impl Default for ErrorMsg {
    fn default() -> Self {
        Self {
            on: false,
            msg: "".to_string(),
        }
    }
}

/// Info message to show in the UI
#[derive(Clone)]
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

impl Default for InfoMsg {
    fn default() -> Self {
        Self {
            on: false,
            msg: "".to_string(),
        }
    }
}

/// Shared State for some GUI components
#[derive(Clone)]
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

    /// Error message to show in the UI
    pub err_msg: ErrorMsg,

    /// Info message to show in the UI
    pub info_msg: InfoMsg,
}

impl Default for SharedUiState {
    fn default() -> Self {
        Self {
            networks_on: false,
            wallet_popup: (false, "New"),
            export_key_ui: false,
            exported_key_window: (false, "".to_string()),
            err_msg: ErrorMsg::default(),
            info_msg: InfoMsg::default(),
        }
    }
}