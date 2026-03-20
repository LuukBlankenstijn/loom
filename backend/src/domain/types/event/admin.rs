use derive_more::derive::From;

use crate::domain::event::station::StationCommand;

#[derive(Clone, Debug)]
pub struct CommandOutput {
    pub id: String,
    pub output: String,
}

#[derive(Clone, Debug, From)]
pub enum AdminEvent {
    Station((Vec<String>, StationCommand)),
}

#[derive(Clone, Debug, From)]
pub enum AdminCommand {
    Command(CommandOutput),
}
