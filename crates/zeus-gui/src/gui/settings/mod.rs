pub struct SettingsUi {

    /// Network settings UI on/off
    pub networks_on: bool,

    /// New/Import Wallet UI on/off
    /// 
    /// (on/off, "New"/"Import Wallet")
    pub wallet_popup: (bool, &'static str),

}




impl Default for SettingsUi {
    fn default() -> Self {
        Self {
            networks_on: false,
            wallet_popup: (false, "New")
        }
    }
}
