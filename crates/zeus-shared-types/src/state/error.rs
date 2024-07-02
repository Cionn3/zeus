/// An Error message to show in the UI
#[derive(Clone, Default)]
pub struct ErrorMsg {
    pub on: bool,

    pub msg: String,
}

impl ErrorMsg {
    pub fn new<T>(on: bool, msg: T) -> Self where T: ToString {
        Self {
            on,
            msg: msg.to_string(),
        }
    }
}