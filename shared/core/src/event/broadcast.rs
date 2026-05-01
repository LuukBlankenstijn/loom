use derive_more::derive::From;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct StationConnectionState {
    pub ip: String,
    pub connected: bool,
    pub logged_in: bool,
}

#[derive(Debug, Clone)]
pub struct StationAssignment {
    pub seat_id: Uuid,
    pub station_ip: Option<String>,
    pub team_name: Option<String>,
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

impl From<(Uuid, Option<String>)> for StationAssignment {
    fn from(value: (Uuid, Option<String>)) -> Self {
        Self {
            seat_id: value.0,
            station_ip: value.1,
            team_name: None,
        }
    }
}

#[derive(Debug, Clone, From)]
pub enum BroadcastEvent {
    Connection(Vec<StationConnectionState>),
    Assignment(Vec<StationAssignment>),
}

/// Subscribe filter — which event types to receive.
#[derive(Debug, Clone, Copy)]
pub enum BroadcastType {
    ConnectionState,
    AssignmentState,
}
