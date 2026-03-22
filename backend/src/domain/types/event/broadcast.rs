use derive_more::derive::From;
use uuid::Uuid;

use crate::domain::StationAssignment;

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

impl From<(String, Option<Uuid>)> for StationAssignment {
    fn from(value: (String, Option<Uuid>)) -> Self {
        Self {
            station_ip: value.0,
            seat_id: value.1,
        }
    }
}

#[derive(Debug, Clone, From)]
pub enum BroadcastEvent {
    Connection(Vec<StationConnectionState>),
    Assignment(Vec<StationAssignment>),
}
