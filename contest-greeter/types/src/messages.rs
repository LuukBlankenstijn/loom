pub enum UiMessage {
    SetWallpaper(Option<String>),
    SetError(String),
}

pub enum GreeterMessage {
    Login(String, String),
    StartSession(Option<String>),
}
