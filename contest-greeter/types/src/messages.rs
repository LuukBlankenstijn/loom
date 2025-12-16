use chrono::{DateTime, Local};

pub enum UiMessage {
    SetWallpaper(Option<String>),
    SetError(String),
    SetCountdownEndtime { end_time: Option<DateTime<Local>> },
}

pub enum GreeterMessage {
    LoginWithCreds(String, String),
    Login(),
    StartSession(Option<String>),
}
