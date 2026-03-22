use loom_core::event::station::{CustomCommandOutput, StationCommand, StationEvent};
use loom_rpc::command::v1 as command_pb;
use loom_rpc::station::v1 as pb;
use tonic::Status;

use crate::{IntoProto, TryIntoCore};

impl TryIntoCore<StationEvent> for pb::StationEvent {
    type Error = Status;

    fn try_into_core(self) -> Result<StationEvent, Status> {
        let message = self
            .message
            .ok_or_else(|| Status::invalid_argument("inner message is not set"))?;
        let event = match message {
            pb::station_event::Message::LoggedOut(_) => StationEvent::LoggedOut,
            pb::station_event::Message::LoggedIn(_) => StationEvent::LoggedIn,
            pb::station_event::Message::CommandOutput(o) => {
                StationEvent::CustomCommand(CustomCommandOutput {
                    id: o.id,
                    admin_id: o.admin_id,
                    output: o.output,
                })
            }
        };
        Ok(event)
    }
}

impl IntoProto<pb::StationCommand> for StationCommand {
    fn into_proto(self) -> pb::StationCommand {
        let inner = match self {
            StationCommand::SyncWallpaper => {
                pb::station_command::Message::SyncWallpaper(())
            }
            StationCommand::SyncContestUrl => {
                pb::station_command::Message::SyncContestUrl(())
            }
            StationCommand::Login => {
                pb::station_command::Message::Login(command_pb::LoginCommand {})
            }
            StationCommand::Logout => {
                pb::station_command::Message::Logout(command_pb::LogoutCommand {})
            }
            StationCommand::LoginWithCredentials { username, password } => {
                pb::station_command::Message::LoginWithCredentials(
                    command_pb::LoginWithCredentialsCommand { username, password },
                )
            }
            StationCommand::CustomCommand(c) => {
                pb::station_command::Message::CustomCommand(pb::CustomCommand {
                    id: c.id,
                    admin_id: c.admin_id,
                    command: c.command,
                })
            }
        };
        pb::StationCommand {
            message: Some(inner),
        }
    }
}
