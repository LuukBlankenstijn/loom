use loom_rpc::broadcast::v1 as pb;
use uuid::Uuid;

pub(crate) mod types;

impl From<types::BroadcastType> for pb::BroadcastType {
    fn from(value: types::BroadcastType) -> Self {
        match value {
            types::BroadcastType::ConnectionState => pb::BroadcastType::ConnectionState,
            types::BroadcastType::AssignmentState => pb::BroadcastType::StationAssignments,
        }
    }
}

impl From<pb::StationState> for types::StationState {
    fn from(value: pb::StationState) -> Self {
        Self {
            ip: value.ip,
            connected: value.connected,
            logged_in: value.logged_in,
        }
    }
}

impl TryFrom<pb::StationAssignment> for types::StationAssignment {
    type Error = String;

    fn try_from(value: pb::StationAssignment) -> Result<Self, Self::Error> {
        let seat_id = value
            .seat_id
            .map(|i| Uuid::parse_str(&i).map_err(|_| "Invalid uuid".to_string()))
            .transpose()?;

        Ok(Self {
            ip: value.ip,
            seat_id,
        })
    }
}

impl TryFrom<pb::BroadcastEvent> for types::BroadcastEvent {
    type Error = String;

    fn try_from(value: pb::BroadcastEvent) -> Result<Self, Self::Error> {
        let event = match value.message.ok_or("No message found")? {
            pb::broadcast_event::Message::StationsState(station_state_update) => {
                types::BroadcastEvent::ConnectionState(
                    station_state_update
                        .state
                        .into_iter()
                        .map(types::StationState::from)
                        .collect(),
                )
            }
            pb::broadcast_event::Message::StationAssignments(station_assignment_update) => {
                types::BroadcastEvent::AssignmentState(
                    station_assignment_update
                        .updates
                        .into_iter()
                        .filter_map(|u| types::StationAssignment::try_from(u).ok())
                        .collect(),
                )
            }
        };
        Ok(event)
    }
}
