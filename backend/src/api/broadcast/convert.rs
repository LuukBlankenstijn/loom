use loom_rpc::broadcast::v1 as pb;

use crate::domain::event::broadcast::{BroadcastEvent, StationConnectionState};

impl From<&StationConnectionState> for pb::StationState {
    fn from(value: &StationConnectionState) -> Self {
        Self {
            ip: value.ip.clone(),
            connected: value.connected,
            logged_in: value.logged_in,
        }
    }
}

impl From<BroadcastEvent> for pb::BroadcastEvent {
    fn from(value: BroadcastEvent) -> Self {
        let inner = match value {
            BroadcastEvent::State(stations_state) => {
                pb::broadcast_event::Message::StationsState(pb::StationStateUpdate {
                    state: stations_state.iter().map(Into::into).collect(),
                })
            }
        };
        pb::BroadcastEvent {
            message: Some(inner),
        }
    }
}
