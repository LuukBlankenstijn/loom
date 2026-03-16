use derive_more::derive::From;

use crate::domain::event::station::CommandOutput;

#[derive(Debug, Clone)]
pub struct StationConnectionState {
    pub ip: String,
    pub logged_in: bool,
}

#[derive(Debug, Clone)]
pub struct StationsState(pub Vec<StationConnectionState>);

#[derive(Debug, Clone, From)]
pub enum BroadcastEvent {
    State(StationsState),
    Command(CommandOutput),
}
