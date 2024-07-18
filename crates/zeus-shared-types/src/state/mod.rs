pub mod shared_ui;
pub mod swap_ui;
pub mod info;
pub mod error;
pub mod data;


pub use shared_ui::{SharedUiState, SHARED_UI_STATE};
pub use swap_ui::{SwapUIState, SWAP_UI_STATE, SelectedCurrency};

/// Indicates whether we should show a UI or not
#[derive(Clone, Default)]
pub enum UiState {
    OPEN,
    #[default]
    CLOSE
}

impl UiState {
    /// Its closed, we should not show the UI
    pub fn is_close(&self) -> bool {
        matches!(self, UiState::CLOSE)
    }

    /// Its open, we should show the UI
    pub fn is_open(&self) -> bool {
        matches!(self, UiState::OPEN)
    }
}