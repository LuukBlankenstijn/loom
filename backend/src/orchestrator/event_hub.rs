use std::sync::Arc;
use std::{collections::HashMap, sync::RwLock};

use tokio::sync::broadcast;
use tokio::sync::mpsc;

use crate::domain::event::admin::AdminCommand;
use crate::orchestrator::types::Registration;
use crate::{
    domain::{
        self,
        event::{broadcast::BroadcastEvent, station::StationCommand},
    },
    error::AppError,
};

pub struct EventHub {
    stations: RwLock<HashMap<String, mpsc::UnboundedSender<StationCommand>>>,
    admins: RwLock<HashMap<String, mpsc::UnboundedSender<AdminCommand>>>,

    broadcast: broadcast::Sender<BroadcastEvent>,
}

impl EventHub {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(32);
        Self {
            broadcast: sender,
            stations: Default::default(),
            admins: Default::default(),
        }
    }
}

impl crate::orchestrator::types::EventHub for EventHub {
    fn register_admin(self: &Arc<Self>, id: &str) -> Result<Registration<AdminCommand>, AppError> {
        let mut admins = self.admins.write().unwrap();
        if admins.contains_key(id) {
            return Err(crate::error::AppError::FailedPrecondition(
                "admin already connected".into(),
            ));
        }
        let (command_tx, command_rx) = mpsc::unbounded_channel();

        admins.insert(id.to_string(), command_tx);
        tracing::debug!(id, "[HUB] registered admin");

        let hub = Arc::clone(self);
        let id_owned = id.to_string();
        let cleanup = move || {
            let mut admins = hub.admins.write().unwrap();
            admins.remove(&id_owned);
            tracing::debug!(id = %id_owned, "[HUB] deregistered admin");
        };

        Ok(Registration::new(command_rx, Some(Box::new(cleanup))))
    }

    fn publish_admin_command(&self, command: AdminCommand, ids: &[&str]) {
        let admins = self.admins.read().unwrap();
        let id_iter: Box<dyn Iterator<Item = &str>> = if !ids.is_empty() {
            Box::new(ids.iter().copied())
        } else {
            Box::new(admins.keys().map(|s| s.as_str()))
        };
        for id in id_iter {
            if let Some(sender) = admins.get(id) {
                // Ignore error
                let _ = sender.send(command.clone());
            }
        }
    }
    fn register_station(
        self: &Arc<Self>,
        ip: &str,
    ) -> Result<Registration<StationCommand>, AppError> {
        let mut stations = self.stations.write().unwrap();
        if stations.contains_key(ip) {
            return Err(crate::error::AppError::FailedPrecondition(
                "station already connected".into(),
            ));
        }
        let (command_tx, command_rx) = mpsc::unbounded_channel();

        stations.insert(ip.to_string(), command_tx);
        tracing::debug!(ip, "[HUB] registered station");

        let hub = Arc::clone(self);
        let ip_owned = ip.to_string();
        let cleanup = move || {
            let mut stations = hub.stations.write().unwrap();
            stations.remove(&ip_owned);
            tracing::debug!(ip = %ip_owned, "[HUB] deregistered station");
        };

        Ok(Registration::new(command_rx, Some(Box::new(cleanup))))
    }

    fn publish_station_command(&self, command: StationCommand, ips: &[&str]) {
        let stations = self.stations.read().unwrap();
        let ip_iter: Box<dyn Iterator<Item = &str>> = if !ips.is_empty() {
            Box::new(ips.iter().copied())
        } else {
            Box::new(stations.keys().map(|s| s.as_str()))
        };
        for ip in ip_iter {
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
