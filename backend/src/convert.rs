use chrono::{DateTime, Utc};
use loom_rpc::admin::v1 as admin_pb;
use loom_rpc::command::v1 as command_pb;
use loom_rpc::map::v1 as map_pb;
use loom_rpc::stations::v1 as stations_pb;
use prost_types::Timestamp;
use uuid::Uuid;

use crate::domain;
use crate::hub::{HubStateEvent, StationCommand};

// ── Timestamp helpers ──────────────────────────────────────────────

pub fn to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

// ── domain::Station → proto Station ────────────────────────────────

impl From<domain::Station> for admin_pb::Station {
    fn from(s: domain::Station) -> Self {
        Self {
            id: s.id,
            ip: s.ip,
            connected_at: Some(to_timestamp(s.connected_at)),
            diconnected_at: s.disconnected_at.map(to_timestamp),
        }
    }
}

// ── domain::Contest → proto Contest ────────────────────────────────

impl From<domain::Contest> for admin_pb::Contest {
    fn from(c: domain::Contest) -> Self {
        Self {
            id: c.id,
            name: c.name,
            start_time: Some(to_timestamp(c.start_time)),
            end_time: Some(to_timestamp(c.end_time)),
            map_id: None,
        }
    }
}

// ── domain::Team → proto Team ──────────────────────────────────────

impl From<domain::Team> for admin_pb::Team {
    fn from(t: domain::Team) -> Self {
        Self {
            id: t.id,
            ip: t.ip,
            name: t.name,
        }
    }
}

// ── domain::Map → proto Map ────────────────────────────────────────

impl From<domain::Map> for map_pb::Map {
    fn from(m: domain::Map) -> Self {
        Self {
            id: m.id,
            name: m.name,
        }
    }
}

// ── domain::Rotation ↔ proto Rotation ──────────────────────────────

impl From<domain::Rotation> for map_pb::Rotation {
    fn from(r: domain::Rotation) -> Self {
        match r {
            domain::Rotation::R0 => Self::Rotation0,
            domain::Rotation::R90 => Self::Rotation90,
            domain::Rotation::R180 => Self::Rotation180,
            domain::Rotation::R270 => Self::Rotation270,
        }
    }
}

impl From<map_pb::Rotation> for domain::Rotation {
    fn from(r: map_pb::Rotation) -> Self {
        match r {
            map_pb::Rotation::Rotation90 => Self::R90,
            map_pb::Rotation::Rotation180 => Self::R180,
            map_pb::Rotation::Rotation270 => Self::R270,
            _ => Self::R0,
        }
    }
}

// ── domain::FullMap → proto MapResponse ────────────────────────────

impl From<domain::FullMap> for admin_pb::MapResponse {
    fn from(m: domain::FullMap) -> Self {
        let mut elements = Vec::with_capacity(m.walls.len() + m.doors.len() + m.tables.len());

        for w in m.walls {
            elements.push(map_pb::Element {
                element: Some(map_pb::element::Element::Wall(map_pb::Wall {
                    id: w.id.to_string(),
                    start: Some(map_pb::Location {
                        x: w.x_start,
                        y: w.y_start,
                    }),
                    end: Some(map_pb::Location {
                        x: w.x_end,
                        y: w.y_end,
                    }),
                })),
            });
        }
        for d in m.doors {
            elements.push(map_pb::Element {
                element: Some(map_pb::element::Element::Door(map_pb::Door {
                    id: d.id.to_string(),
                    location: Some(map_pb::Location { x: d.x, y: d.y }),
                    rotation: map_pb::Rotation::from(d.rotation) as i32,
                })),
            });
        }
        for t in m.tables {
            elements.push(map_pb::Element {
                element: Some(map_pb::element::Element::Table(map_pb::Table {
                    id: t.id.to_string(),
                    location: Some(map_pb::Location { x: t.x, y: t.y }),
                    rotation: map_pb::Rotation::from(t.rotation) as i32,
                })),
            });
        }

        Self {
            map: Some(map_pb::Map {
                id: m.id,
                name: m.name,
            }),
            elements,
        }
    }
}

// ── proto Element → domain types (for UpdateMap) ───────────────────

impl TryFrom<&map_pb::Element> for domain::Wall {
    type Error = tonic::Status;

    fn try_from(el: &map_pb::Element) -> Result<Self, Self::Error> {
        match &el.element {
            Some(map_pb::element::Element::Wall(w)) => {
                let id = Uuid::parse_str(&w.id)
                    .map_err(|_| tonic::Status::invalid_argument("invalid wall uuid"))?;
                let start = w.start.as_ref().unwrap_or(&map_pb::Location { x: 0, y: 0 });
                let end = w.end.as_ref().unwrap_or(&map_pb::Location { x: 0, y: 0 });
                Ok(Self {
                    id,
                    x_start: start.x,
                    y_start: start.y,
                    x_end: end.x,
                    y_end: end.y,
                })
            }
            _ => Err(tonic::Status::invalid_argument("expected wall element")),
        }
    }
}

impl TryFrom<&map_pb::Element> for domain::Door {
    type Error = tonic::Status;

    fn try_from(el: &map_pb::Element) -> Result<Self, Self::Error> {
        match &el.element {
            Some(map_pb::element::Element::Door(d)) => {
                let id = Uuid::parse_str(&d.id)
                    .map_err(|_| tonic::Status::invalid_argument("invalid door uuid"))?;
                let loc = d
                    .location
                    .as_ref()
                    .unwrap_or(&map_pb::Location { x: 0, y: 0 });
                let rotation = map_pb::Rotation::try_from(d.rotation)
                    .unwrap_or(map_pb::Rotation::Rotation0)
                    .into();
                Ok(Self {
                    id,
                    x: loc.x,
                    y: loc.y,
                    rotation,
                })
            }
            _ => Err(tonic::Status::invalid_argument("expected door element")),
        }
    }
}

impl TryFrom<&map_pb::Element> for domain::Table {
    type Error = tonic::Status;

    fn try_from(el: &map_pb::Element) -> Result<Self, Self::Error> {
        match &el.element {
            Some(map_pb::element::Element::Table(t)) => {
                let id = Uuid::parse_str(&t.id)
                    .map_err(|_| tonic::Status::invalid_argument("invalid table uuid"))?;
                let loc = t
                    .location
                    .as_ref()
                    .unwrap_or(&map_pb::Location { x: 0, y: 0 });
                let rotation = map_pb::Rotation::try_from(t.rotation)
                    .unwrap_or(map_pb::Rotation::Rotation0)
                    .into();
                Ok(Self {
                    id,
                    x: loc.x,
                    y: loc.y,
                    rotation,
                })
            }
            _ => Err(tonic::Status::invalid_argument("expected table element")),
        }
    }
}

// ── StationCommand → proto ServerMessage ───────────────────────────

impl From<StationCommand> for stations_pb::ServerMessage {
    fn from(cmd: StationCommand) -> Self {
        use stations_pb::server_message::Message;
        let message = match cmd {
            StationCommand::SetWallpaperSource(s) => Message::SetWallpaperSource(s),
            StationCommand::SetContestUrl(u) => Message::SetContestUrl(u),
            StationCommand::Login => Message::Login(command_pb::LoginCommand {}),
            StationCommand::Logout => Message::Logout(command_pb::LogoutCommand {}),
            StationCommand::LoginWithCredentials { username, password } => {
                Message::LoginWithCredentials(command_pb::LoginWithCredentialsCommand {
                    username,
                    password,
                })
            }
            StationCommand::CustomCommand { id, command } => {
                Message::CustomCommand(command_pb::CustomCommand { id, command })
            }
        };
        Self {
            message: Some(message),
        }
    }
}

// ── proto ClientMessage → action enum for the station handler ──────

#[derive(Clone, Debug)]
pub struct CommandOutput {
    id: String,
    output: String,
}

/// What a station client is telling us.
pub enum ClientAction {
    LoggedIn,
    LoggedOut,
    Command(CommandOutput),
}

impl TryFrom<stations_pb::ClientMessage> for ClientAction {
    type Error = tonic::Status;

    fn try_from(msg: stations_pb::ClientMessage) -> Result<Self, Self::Error> {
        use stations_pb::client_message::Message;
        match msg.message {
            Some(Message::LoggedIn(_)) => Ok(Self::LoggedIn),
            Some(Message::LoggedOut(_)) => Ok(Self::LoggedOut),
            Some(Message::CommandOutput(o)) => Ok(Self::Command(CommandOutput {
                id: o.id,
                output: o.output,
            })),
            None => Err(tonic::Status::invalid_argument("empty client message")),
        }
    }
}

impl From<CommandOutput> for admin_pb::SubscribtionMessage {
    fn from(value: CommandOutput) -> Self {
        Self {
            message: Some(admin_pb::subscribtion_message::Message::CommandOutput(
                command_pb::CustomCommandOutput {
                    id: value.id,
                    output: value.output,
                },
            )),
        }
    }
}

impl From<HubStateEvent> for admin_pb::SubscribtionMessage {
    fn from(value: HubStateEvent) -> Self {
        Self {
            message: Some(admin_pb::subscribtion_message::Message::StatusUpdate(
                admin_pb::StationsState {
                    status: value
                        .0
                        .iter()
                        .map(|s| admin_pb::StationStatus {
                            ip: s.ip.clone(),
                            logged_in: s.logged_in,
                        })
                        .collect(),
                },
            )),
        }
    }
}

impl From<admin_pb::client_command::Command> for StationCommand {
    fn from(value: admin_pb::client_command::Command) -> Self {
        match value {
            admin_pb::client_command::Command::LoginWithCredentials(
                login_with_credentials_comman,
            ) => Self::LoginWithCredentials {
                username: login_with_credentials_comman.username,
                password: login_with_credentials_comman.password,
            },
            admin_pb::client_command::Command::Login(_) => Self::Login,
            admin_pb::client_command::Command::Logout(_) => Self::Logout,
            admin_pb::client_command::Command::Custom(custom_command) => Self::CustomCommand {
                id: custom_command.id,
                command: custom_command.command,
            },
        }
    }
}
