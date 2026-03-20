use crate::{
    domain::{
        self,
        event::{
            admin::{AdminCommand, AdminEvent},
            station::{Command, StationCommand},
        },
    },
    error::AppError,
};
use loom_rpc::admin::v1 as pb;

impl From<domain::Station> for pb::Station {
    fn from(s: domain::Station) -> Self {
        Self { ip: s.ip }
    }
}

impl TryFrom<pb::AdminEvent> for AdminEvent {
    type Error = AppError;

    fn try_from(value: pb::AdminEvent) -> Result<Self, Self::Error> {
        let command = value.command.ok_or(AppError::InvalidArgument(
            "inner command is empty".to_string(),
        ))?;
        let event = match command {
            pb::admin_event::Command::LoginWithCredentials(command) => {
                StationCommand::LoginWithCredentials {
                    username: command.username,
                    password: command.password,
                }
            }
            pb::admin_event::Command::Login(_) => StationCommand::Login,
            pb::admin_event::Command::Logout(_) => StationCommand::Logout,
            pb::admin_event::Command::Custom(custom_command) => Command {
                id: custom_command.id,
                admin_id: custom_command.admin_id,
                command: custom_command.command,
            }
            .into(),
        };

        Ok((value.ips, event).into())
    }
}

impl From<AdminCommand> for pb::CustomCommandOutput {
    fn from(value: AdminCommand) -> Self {
        match value {
            AdminCommand::Command(command_output) => pb::CustomCommandOutput {
                id: command_output.id,
                output: command_output.output,
            },
        }
    }
}
