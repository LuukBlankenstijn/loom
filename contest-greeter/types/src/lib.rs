pub mod ui {
    #[derive(Debug)]
    pub enum CoreMessage {
        Login(String, String),
    }
}

pub mod core {
    #[derive(Debug)]
    pub enum UiMessage {
        SetWallpaper(Option<String>),
        SetError(String),
    }
}
