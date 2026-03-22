use std::sync::Arc;

use derive_more::derive::Constructor;
use loom_core::event::{
    admin::AdminCommand,
    broadcast::{BroadcastEvent, StationConnectionState},
    station::StationCommand,
};
use tokio::sync::{broadcast, mpsc};

use crate::error::AppError;

#[derive(Constructor)]
pub struct Registration<T> {
    pub receiver: mpsc::UnboundedReceiver<T>,
    cleanup: Option<Box<dyn FnOnce() + Send>>,
}

impl<T> Drop for Registration<T> {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

impl<T> Registration<T> {
    // add extra cleanup. Added will run after currente cleanup
    pub fn add_cleanup(&mut self, new_cleanup: Box<dyn FnOnce() + Send>) {
        if let Some(cleanup) = self.cleanup.take() {
            let replacement = move || {
                cleanup();
                new_cleanup();
            };
            self.cleanup = Some(Box::new(replacement))
        }
    }
}

pub trait EventHub: Sync + Send {
    // station
    fn register_station(
        self: &Arc<Self>,
        ip: &str,
    ) -> Result<Registration<StationCommand>, AppError>;
    fn publish_station_command(&self, command: StationCommand, ips: &[&str]);

    // admin
    fn register_admin(self: &Arc<Self>, ip: &str) -> Result<Registration<AdminCommand>, AppError>;
    fn publish_admin_command(&self, command: AdminCommand, ips: &[&str]);

    // broadcast
    fn subscribe_broadcast(&self) -> broadcast::Receiver<BroadcastEvent>;
    fn broadcast(&self, event: BroadcastEvent);
}

pub type StateChangeHook = Box<dyn Fn(StationConnectionState) + Send + Sync>;

pub trait StationStateStore: Send + Sync {
    fn get_state(&self) -> Vec<StationConnectionState>;

    fn connect(&self, ip: &str);
    fn disconnect(&self, ip: &str);
    fn login(&self, ip: &str);
    fn logout(&self, ip: &str);

    fn set_on_change_hook(&mut self, hook: StateChangeHook);
}
