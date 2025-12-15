pub enum UiMessage {
    SetWallpaper(Option<String>),
    SetError(String),
}

pub enum GreeterMessage {
    LoginWithCreds(String, String),
    Login(),
    StartSession(Option<String>),
}
