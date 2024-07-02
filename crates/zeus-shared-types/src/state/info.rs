/// An Info message to show in the UI
#[derive(Clone, Default)]
pub struct InfoMsg {
    pub on: bool,

    pub msg: String,
}

impl InfoMsg {
    pub fn new<T>(on: bool, msg: T) -> Self where T: ToString {
        Self {
            on,
            msg: msg.to_string(),
        }
    }
}