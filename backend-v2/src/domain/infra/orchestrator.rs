use async_trait::async_trait;
use std::pin::Pin;

use futures::Stream;

use crate::{
    domain::event::{LoomEvent, broadcast::BroadcastEvent, station::StationCommand},
    error::AppError,
};

pub type StationCommandStream =
    Pin<Box<dyn Stream<Item = Result<StationCommand, AppError>> + Send>>;
pub type BroadcastEventStream =
    Pin<Box<dyn Stream<Item = Result<BroadcastEvent, AppError>> + Send>>;

#[async_trait]
pub trait Orchestrator: Send + Sync {
    fn handle_event(&self, event: LoomEvent);
    fn sync_wallpaper(&self, ips: &[&str]);
    fn sync_api_url(&self, ips: &[&str], contest_id: String);
    async fn register_station(&self, ip: &str) -> Result<StationCommandStream, AppError>;
    fn subscribe_broadcast(&self) -> BroadcastEventStream;
}
