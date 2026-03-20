use loom_rpc::command::v1 as command_pb;
use loom_rpc::station::v1::{self as pb};

use crate::{
    domain::event::station::{CommandOutput, StationCommand, StationEvent},
    error::AppError,
};

impl TryFrom<pb::StationEvent> for StationEvent {
    type Error = AppError;

    fn try_from(value: pb::StationEvent) -> Result<Self, Self::Error> {
        let message = value.message.ok_or(AppError::InvalidArgument(
            "innner message is not set".to_string(),
        ))?;
        let event = match message {
            pb::station_event::Message::LoggedOut(_) => Self::LoggedOut,
            pb::station_event::Message::LoggedIn(_) => Self::LoggedIn,
            pb::station_event::Message::CommandOutput(custom_command_output) => {
                Self::Command(CommandOutput {
                    id: custom_command_output.id,
                    admin_id: custom_command_output.admin_id,
                    output: custom_command_output.output,
                })
            }
        };
        Ok(event)
    }
}

impl From<StationCommand> for pb::StationCommand {
    fn from(value: StationCommand) -> Self {
        let inner = match value {
            StationCommand::SyncWallpaper => pb::station_command::Message::SyncWallpaper(()),
            StationCommand::SyncContestUrl => pb::station_command::Message::SyncContestUrl(()),
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
            StationCommand::CustomCommand(command) => {
                pb::station_command::Message::CustomCommand(pb::CustomCommand {
                    id: command.id,
                    admin_id: command.admin_id,
                    command: command.command,
                })
            }
        };

        pb::StationCommand {
            message: Some(inner),
        }
    }
}
