use loom_core::event::broadcast::{
    BroadcastEvent, BroadcastType, StationAssignment, StationConnectionState,
};
use loom_rpc::broadcast::v1 as pb;
use uuid::Uuid;

use crate::{FromProto, IntoProto, TryIntoCore};

impl IntoProto<pb::BroadcastType> for BroadcastType {
    fn into_proto(self) -> pb::BroadcastType {
        match self {
            BroadcastType::ConnectionState => pb::BroadcastType::ConnectionState,
            BroadcastType::AssignmentState => pb::BroadcastType::StationAssignments,
        }
    }
}

impl FromProto<pb::StationState> for StationConnectionState {
    fn from_proto(value: pb::StationState) -> Self {
        Self {
            ip: value.ip,
            connected: value.connected,
            logged_in: value.logged_in,
        }
    }
}

impl IntoProto<pb::StationState> for StationConnectionState {
    fn into_proto(self) -> pb::StationState {
        pb::StationState {
            ip: self.ip,
            connected: self.connected,
            logged_in: self.logged_in,
        }
    }
}

impl TryIntoCore<StationAssignment> for pb::StationAssignment {
    type Error = String;

    fn try_into_core(self) -> Result<StationAssignment, String> {
        let seat_id = self
            .seat_id
            .map(|i| Uuid::parse_str(&i).map_err(|_| "invalid uuid".to_string()))
            .transpose()?;
        Ok(StationAssignment {
            station_ip: self.ip,
            seat_id,
        })
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

impl TryIntoCore<BroadcastEvent> for pb::BroadcastEvent {
    type Error = String;

    fn try_into_core(self) -> Result<BroadcastEvent, String> {
        let event = match self.message.ok_or("no message found")? {
            pb::broadcast_event::Message::StationsState(update) => {
                BroadcastEvent::Connection(
                    update
                        .state
                        .into_iter()
                        .map(StationConnectionState::from_proto)
                        .collect(),
                )
            }
            pb::broadcast_event::Message::StationAssignments(update) => {
                BroadcastEvent::Assignment(
                    update
                        .updates
                        .into_iter()
                        .filter_map(|u| u.try_into_core().ok())
                        .collect(),
                )
            }
        };
        Ok(event)
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
