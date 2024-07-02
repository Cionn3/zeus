pub mod state;

pub use state::{
    data::{ AppData, NETWORKS, TxSettings },
    swap_ui::{ SWAP_UI_STATE, SelectedCurrency, SwapUIState },
    shared_ui::SHARED_UI_STATE,
    SharedUiState,
    error::ErrorMsg,
    info::InfoMsg
};
