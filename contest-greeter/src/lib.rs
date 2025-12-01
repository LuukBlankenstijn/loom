#[derive(Debug)]
pub enum UICoreCommand {
    Login(String, String),
}

#[derive(Debug)]
pub enum CoreUICommand {
    SetWallpaper(Option<String>),
    SetError(String),
}
