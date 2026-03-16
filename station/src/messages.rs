#[derive(Clone, Debug)]
pub enum Message {
    LoggedIn,
    LoggedOut,
    SyncWallpaper,
    SetContestUrl,
    Login,
    LoginWithCredentials(String, String),
    Logout,
    RequestLoginStatus,
    RunCommand { id: String, command: String },
    CommandOutput { id: String, output: String },
}
