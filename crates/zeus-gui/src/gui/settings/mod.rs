pub struct SettingsUi {

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

}




impl Default for SettingsUi {
    fn default() -> Self {
        Self {
            networks_on: false,
            wallet_popup: (false, "New"),
            export_key_ui: false,
            exported_key_window: (false, "".to_string()),
        }
    }
}
