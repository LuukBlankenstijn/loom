#[derive(Clone, Debug)]
pub struct CommandOutput {
    id: String,
    output: String,
}

// Events comming from a statino
#[derive(Clone, Debug)]
pub enum StationEvent {
    LoggedIn,
    LoggedOut,
    Command(CommandOutput),
}

// Command going to a station
#[derive(Debug, Clone)]
pub enum StationCommand {
    SyncWallpaper,
    SetContestUrl(String),
    Login,
    Logout,
    LoginWithCredentials { username: String, password: String },
    CustomCommand { id: String, command: String },
}
