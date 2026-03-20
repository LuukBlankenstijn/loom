use derive_more::derive::From;

#[derive(Debug, Clone)]
pub struct StationConnectionState {
    pub ip: String,
    pub connected: bool,
    pub logged_in: bool,
}

impl From<(String, bool, bool)> for StationConnectionState {
    fn from(value: (String, bool, bool)) -> Self {
        Self {
            ip: value.0,
            connected: value.1,
            logged_in: value.2,
        }
    }
}

#[derive(Debug, Clone, From)]
pub enum BroadcastEvent {
    State(Vec<StationConnectionState>),
}
