use loom_core::event::broadcast::{BroadcastEvent, StationAssignment, StationConnectionState};
use loom_rpc::broadcast::v1 as pb;

use crate::IntoProto;

impl IntoProto<pb::StationState> for StationConnectionState {
    fn into_proto(self) -> pb::StationState {
        pb::StationState {
            ip: self.ip,
            connected: self.connected,
            logged_in: self.logged_in,
        }
    }
}

impl IntoProto<pb::StationAssignment> for StationAssignment {
    fn into_proto(self) -> pb::StationAssignment {
        pb::StationAssignment {
            ip: self.station_ip,
            seat_id: self.seat_id.map(|uuid| uuid.to_string()),
        }
    }
}

impl IntoProto<pb::BroadcastEvent> for BroadcastEvent {
    fn into_proto(self) -> pb::BroadcastEvent {
        let inner = match self {
            BroadcastEvent::Connection(states) => {
                pb::broadcast_event::Message::StationsState(pb::StationStateUpdate {
                    state: states.into_iter().map(IntoProto::into_proto).collect(),
                })
            }
            BroadcastEvent::Assignment(assignments) => {
                pb::broadcast_event::Message::StationAssignments(pb::StationAssignmentUpdate {
                    updates: assignments.into_iter().map(IntoProto::into_proto).collect(),
                })
            }
        };
        pb::BroadcastEvent {
            message: Some(inner),
        }
    }
}
