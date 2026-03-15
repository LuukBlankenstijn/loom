use async_trait::async_trait;
use std::pin::Pin;

use futures::Stream;
use tokio::sync::broadcast::Receiver;

use crate::{
    domain::event::{LoomEvent, broadcast::BroadcastEvent, station::StationCommand},
    error::AppError,
};

pub type EventStream = Pin<Box<dyn Stream<Item = Result<StationCommand, AppError>> + Send>>;

#[async_trait]
pub trait Orchestrator: Send + Sync {
    fn handle_event(&self, event: LoomEvent);
    fn sync_wallpaper(&self, ips: &[&str]);
    fn sync_api_url(&self, ips: &[&str], contest_id: String);
    async fn register_station(&self, ip: &str) -> Result<EventStream, AppError>;
    fn subscribe_broadcast(&self) -> Receiver<BroadcastEvent>;
}
