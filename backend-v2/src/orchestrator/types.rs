use std::sync::Arc;

use derive_more::derive::Constructor;
use tokio::sync::{broadcast, mpsc};

use crate::domain::event::broadcast::StationsState;
use crate::domain::event::{broadcast::BroadcastEvent, station::StationCommand};
use crate::error::AppError;

#[derive(Constructor)]
pub struct StationRegistration {
    pub receiver: mpsc::UnboundedReceiver<StationCommand>,
    cleanup: Option<Box<dyn FnOnce() + Send>>,
}

impl Drop for StationRegistration {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

pub trait EventHub: Sync + Send {
    // station
    fn register_station(self: &Arc<Self>, ip: &str) -> Result<StationRegistration, AppError>;
    fn publish_station_command(&self, command: StationCommand, ips: &[&str]);

    // broadcast
    fn subscribe_broadcast(&self) -> broadcast::Receiver<BroadcastEvent>;
    fn broadcast(&self, event: BroadcastEvent);
}

pub type StateChangeHook = Box<dyn Fn(StationsState) + Send + Sync>;

pub trait StationStateStore: Send + Sync {
    fn get_state(&self) -> StationsState;

    fn connect(&self, ip: &str);
    fn disconnect(&self, ip: &str);
    fn login(&self, ip: &str);
    fn logout(&self, ip: &str);

    fn set_on_change_hook(&mut self, hook: StateChangeHook);
}
