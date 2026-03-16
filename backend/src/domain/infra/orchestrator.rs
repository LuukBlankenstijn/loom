use async_trait::async_trait;
use std::pin::Pin;

use futures::Stream;

use crate::{
    domain::event::{
        LoomEvent,
        broadcast::{BroadcastEvent, StationsState},
        station::StationCommand,
    },
    error::AppError,
};

pub type StationCommandStream =
    Pin<Box<dyn Stream<Item = Result<StationCommand, AppError>> + Send>>;
pub type BroadcastEventStream =
    Pin<Box<dyn Stream<Item = Result<BroadcastEvent, AppError>> + Send>>;

#[async_trait]
pub trait Orchestrator: Send + Sync {
    fn handle_event(&self, event: LoomEvent);
    fn sync_stations(&self, ips: &[&str]);
    async fn register_station(&self, ip: &str) -> Result<StationCommandStream, AppError>;
    fn subscribe_broadcast(&self) -> BroadcastEventStream;
    fn get_state(&self) -> StationsState;
}
