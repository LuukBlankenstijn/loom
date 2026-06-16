use derive_more::derive::From;

#[derive(Clone, Debug)]
pub struct CustomCommandOutput {
    pub id: String,
    pub admin_id: String,
    pub output: String,
}

#[derive(Clone, Debug)]
pub struct CustomCommand {
    pub id: String,
    pub admin_id: String,
    pub command: String,
}

// Events comming from a station
#[derive(Clone, Debug)]
pub enum StationEvent {
    LoggedIn,
    LoggedOut,
    CustomCommand(CustomCommandOutput),
}

// Command going to a station
#[derive(Debug, Clone, From)]
pub enum StationCommand {
    SyncWallpaper,
    SyncContestUrl,
    Login,
    Logout,
    LoginWithCredentials { username: String, password: String },
    CustomCommand(CustomCommand),
    StartRegistrationTool,
    StopRegistrationTool,
}
