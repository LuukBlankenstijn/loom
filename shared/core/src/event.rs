use derive_more::derive::From;

pub mod admin;
pub mod broadcast;
pub mod station;

#[derive(Debug, Clone, From)]
pub enum LoomEvent {
    Station((String, station::StationEvent)),
    Admin(admin::AdminEvent),
}
