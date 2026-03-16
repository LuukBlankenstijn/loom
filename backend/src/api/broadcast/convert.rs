use loom_rpc::broadcast::v1 as pb;
use loom_rpc::command::v1 as command_pb;

use crate::domain::event::broadcast::{BroadcastEvent, StationConnectionState};

impl From<&StationConnectionState> for pb::StationState {
    fn from(value: &StationConnectionState) -> Self {
        Self {
            ip: value.ip.clone(),
            logged_in: value.logged_in,
        }
    }
}

impl From<BroadcastEvent> for pb::BroadcastEvent {
    fn from(value: BroadcastEvent) -> Self {
        let inner = match value {
            BroadcastEvent::State(stations_state) => {
                pb::broadcast_event::Message::StationsState(pb::StationsState {
                    state: stations_state.0.iter().map(Into::into).collect(),
                })
            }
            BroadcastEvent::Command(output) => {
                pb::broadcast_event::Message::CommandOutput(command_pb::CustomCommandOutput {
                    id: output.id,
                    output: output.output,
                })
            }
        };
        pb::BroadcastEvent {
            message: Some(inner),
        }
    }
}
