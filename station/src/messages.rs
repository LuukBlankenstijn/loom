#[derive(Clone, Copy, Debug)]
pub enum DbusEvent {
    LoggedIn,
    LoggedOut,
}

#[derive(Clone, Debug)]
pub enum DbusCommand {
    SetWallpaper(String),
    SetContestUrl(String),
    Login,
    LoginWithCredentials(String, String),
    GetLoginStatus,
}

#[derive(Clone, Debug)]
pub enum RpcCommand {
    LoggedIn,
    LoggedOut,
    CustomCommandOutput(String, String),
}

#[derive(Clone, Debug)]
pub enum RpcEvent {
    SetWallpaper(String),
    SetContestUrl(String),
    Login,
    Logout,
    LoginWithCredentials(String, String),
    CustomCommand(String, String),
    RequestLoginStatus,
}

pub enum CommandRunnerCommand {
    Run { id: Option<String>, command: String },
}

pub enum CommandRunnerEvent {
    Result { id: Option<String>, output: String },
}
