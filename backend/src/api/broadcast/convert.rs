use loom_rpc::broadcast::v1 as pb;

use crate::domain::{
    StationAssignment,
    event::broadcast::{BroadcastEvent, StationConnectionState},
};

impl From<StationConnectionState> for pb::StationState {
    fn from(value: StationConnectionState) -> Self {
        Self {
            ip: value.ip.clone(),
            connected: value.connected,
            logged_in: value.logged_in,
        }
    }
}

impl From<StationAssignment> for pb::StationAssignment {
    fn from(value: StationAssignment) -> Self {
        Self {
            ip: value.station_ip,
            seat_id: value.seat_id.map(|uuid| uuid.to_string()),
        }
    }
}

impl From<BroadcastEvent> for pb::BroadcastEvent {
    fn from(value: BroadcastEvent) -> Self {
        let inner = match value {
            BroadcastEvent::Connection(stations_state) => {
                pb::broadcast_event::Message::StationsState(pb::StationStateUpdate {
                    state: stations_state.into_iter().map(Into::into).collect(),
                })
            }
            BroadcastEvent::Assignment(assignment) => {
                pb::broadcast_event::Message::StationAssignments(pb::StationAssignmentUpdate {
                    updates: assignment.into_iter().map(Into::into).collect(),
                })
            }
        };
        pb::BroadcastEvent {
            message: Some(inner),
        }
    }
}
