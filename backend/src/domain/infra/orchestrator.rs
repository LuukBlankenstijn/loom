use async_trait::async_trait;
use std::pin::Pin;

use futures::Stream;

use crate::{
    domain::event::{
        LoomEvent,
        admin::AdminCommand,
        broadcast::{BroadcastEvent, StationConnectionState},
        station::StationCommand,
    },
    error::AppError,
};

pub type CommandStream<T> = Pin<Box<dyn Stream<Item = Result<T, AppError>> + Send>>;
pub type BroadcastEventStream =
    Pin<Box<dyn Stream<Item = Result<BroadcastEvent, AppError>> + Send>>;

#[async_trait]
pub trait Orchestrator: Send + Sync {
    fn handle_event(&self, event: LoomEvent);
    fn sync_stations(&self, ips: &[&str]);
    async fn register_station(&self, ip: &str) -> Result<CommandStream<StationCommand>, AppError>;
    async fn register_admin(&self, ip: &str) -> Result<CommandStream<AdminCommand>, AppError>;
    fn subscribe_broadcast(&self) -> BroadcastEventStream;
    fn get_state(&self) -> Vec<StationConnectionState>;
}
