use derive_more::derive::From;

use crate::domain::event::station::StationCommand;

#[derive(Clone, Debug)]
pub struct CommandInput {
    pub ips: Vec<String>,
    pub id: String,
    pub command: String,
}

#[derive(Clone, Debug, From)]
pub enum AdminEvent {
    Station((Vec<String>, StationCommand)),
}
