use derive_more::derive::From;

use crate::domain::event::station::StationCommand;

#[derive(Clone, Debug, From)]
pub enum AdminEvent {
    Station((Vec<String>, StationCommand)),
}
