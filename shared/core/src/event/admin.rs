use derive_more::derive::From;

use crate::event::station::StationCommand;

#[derive(Clone, Debug)]
pub struct CommandOutput {
    pub id: String,
    pub output: String,
}

// Event comming from an admin
#[derive(Clone, Debug, From)]
pub enum AdminEvent {
    Station((Vec<String>, StationCommand)),
}

// Events going to an admin
#[derive(Clone, Debug, From)]
pub enum AdminCommand {
    Command(CommandOutput),
}
