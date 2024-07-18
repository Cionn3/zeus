use super::UiState;



/// An Error message to show in the UI
#[derive(Clone, Default)]
pub struct ErrorMsg {
    pub state: UiState,

    pub msg: String,
}

impl ErrorMsg {
    /// Show an ErrorMsg
    /// 
    /// You should have a function called by [eframe::App::update] that checks the [UiState] and paints the Ui for the error message
    pub fn show<T>(&mut self, msg: T) where T: ToString {
        self.state = UiState::OPEN;
        self.msg = msg.to_string();
    }

    /// Close the ErrorMsg
    pub fn close(&mut self) {
        self.state = UiState::CLOSE;
        self.msg.clear();
    }
}