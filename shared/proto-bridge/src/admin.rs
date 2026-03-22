use loom_core::{
    contest::Contest,
    event::admin::{AdminCommand, AdminEvent},
    event::station::{CustomCommand, StationCommand},
    station::Station,
    team::Team,
};
use loom_rpc::admin::v1 as pb;
use tonic::Status;

use crate::{IntoProto, TryIntoCore, to_timestamp};

impl IntoProto<pb::Team> for Team {
    fn into_proto(self) -> pb::Team {
        pb::Team {
            id: self.id,
            ip: self.ip,
            name: self.name,
        }
    }
}

impl IntoProto<pb::Station> for Station {
    fn into_proto(self) -> pb::Station {
        pb::Station { ip: self.ip }
    }
}

impl IntoProto<pb::Contest> for Contest {
    fn into_proto(self) -> pb::Contest {
        pb::Contest {
            id: self.id,
            name: self.name,
            start_time: Some(to_timestamp(self.start_time)),
            end_time: Some(to_timestamp(self.end_time)),
            map_id: None,
        }
    }
}

impl TryIntoCore<AdminEvent> for pb::AdminEvent {
    type Error = Status;

    fn try_into_core(self) -> Result<AdminEvent, Status> {
        let ips = self.ips;
        let command = self
            .command
            .ok_or_else(|| Status::invalid_argument("inner command is empty"))?;
        let station_cmd = match command {
            pb::admin_event::Command::LoginWithCredentials(c) => {
                StationCommand::LoginWithCredentials {
                    username: c.username,
                    password: c.password,
                }
            }
            pb::admin_event::Command::Login(_) => StationCommand::Login,
            pb::admin_event::Command::Logout(_) => StationCommand::Logout,
            pb::admin_event::Command::Custom(c) => CustomCommand {
                id: c.id,
                admin_id: c.admin_id,
                command: c.command,
            }
            .into(),
        };
        Ok((ips, station_cmd).into())
    }
}

impl IntoProto<pb::CustomCommandOutput> for AdminCommand {
    fn into_proto(self) -> pb::CustomCommandOutput {
        match self {
            AdminCommand::Command(c) => pb::CustomCommandOutput {
                id: c.id,
                output: c.output,
            },
        }
    }
}
