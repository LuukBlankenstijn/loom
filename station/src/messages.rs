#[derive(Clone, Debug)]
pub enum Message {
    LoggedIn,
    LoggedOut,
    SyncWallpaper,
    SetContestUrl(String),
    Login,
    LoginWithCredentials(String, String),
    Logout,
    RequestLoginStatus,
    RunCommand { id: String, command: String },
    CommandOutput { id: String, output: String },
}
