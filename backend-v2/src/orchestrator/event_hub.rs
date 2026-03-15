use std::sync::Arc;
use std::{collections::HashMap, sync::RwLock};

use tokio::sync::broadcast;
use tokio::sync::mpsc;

use crate::orchestrator::types::StationRegistration;
use crate::{
    domain::{
        self,
        event::{broadcast::BroadcastEvent, station::StationCommand},
    },
    error::AppError,
};

pub struct EventHub {
    stations: RwLock<HashMap<String, mpsc::UnboundedSender<StationCommand>>>,

    broadcast: broadcast::Sender<BroadcastEvent>,
}

impl EventHub {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(32);
        Self {
            broadcast: sender,
            stations: Default::default(),
        }
    }
}

impl crate::orchestrator::types::EventHub for EventHub {
    fn register_station(self: &Arc<Self>, ip: &str) -> Result<StationRegistration, AppError> {
        let mut stations = self.stations.write().unwrap();
        if stations.contains_key(ip) {
            return Err(crate::error::AppError::FailedPrecondition(
                "station already connected".into(),
            ));
        }
        let (command_tx, command_rx) = mpsc::unbounded_channel();

        stations.insert(ip.to_string(), command_tx);
        tracing::debug!(ip, "[HUB] deregistered station");

        let hub = Arc::clone(self);
        let ip_owned = ip.to_string();
        let cleanup = move || {
            let mut stations = hub.stations.write().unwrap();
            stations.remove(&ip_owned);
            tracing::debug!(ip = %ip_owned, "[HUB] deregistered station");
        };

        Ok(StationRegistration::new(
            command_rx,
            Some(Box::new(cleanup)),
        ))
    }

    fn publish_station_command(&self, command: StationCommand, ips: &[&str]) {
        let stations = self.stations.read().unwrap();
        for &ip in ips {
            if let Some(sender) = stations.get(ip) {
                // Ignore error
                let _ = sender.send(command.clone());
            }
        }
    }

    fn subscribe_broadcast(
        &self,
    ) -> tokio::sync::broadcast::Receiver<domain::event::broadcast::BroadcastEvent> {
        self.broadcast.subscribe()
    }

    fn broadcast(&self, event: domain::event::broadcast::BroadcastEvent) {
        let _ = self.broadcast.send(event);
    }
}
