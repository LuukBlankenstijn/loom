use derive_more::derive::From;

#[derive(Clone, Debug)]
pub struct CommandOutput {
    pub id: String,
    pub admin_id: String,
    pub output: String,
}

#[derive(Clone, Debug)]
pub struct Command {
    pub id: String,
    pub admin_id: String,
    pub command: String,
}

// Events comming from a statino
#[derive(Clone, Debug)]
pub enum StationEvent {
    LoggedIn,
    LoggedOut,
    Command(CommandOutput),
}

// Command going to a station
#[derive(Debug, Clone, From)]
pub enum StationCommand {
    SyncWallpaper,
    SyncContestUrl,
    Login,
    Logout,
    LoginWithCredentials { username: String, password: String },
    CustomCommand(Command),
}
