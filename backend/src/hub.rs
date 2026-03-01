use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use tokio::sync::{broadcast, mpsc};

/// A command for the handler
#[derive(Debug, Clone)]
pub enum StationHandlerCommand {
    /// pass on the contained station comand
    Station(StationCommand),
    /// Sync the client state
    Sync,
}

/// A command sent TO a specific station client.
#[derive(Debug, Clone)]
pub enum StationCommand {
    SetWallpaperSource(String),
    SetContestUrl(String),
    Login,
    Logout,
    LoginWithCredentials { username: String, password: String },
    CustomCommand { id: String, command: String },
}

impl From<StationCommand> for StationHandlerCommand {
    fn from(value: StationCommand) -> Self {
        Self::Station(value)
    }
}

/// A snapshot of one connected station's state.
#[derive(Debug, Clone)]
pub struct ConnectedStation {
    pub ip: String,
    pub connected_at: DateTime<Utc>,
    pub logged_in: bool,
}

/// Every broadcast is a full snapshot of all connected stations.
pub type HubStateEvent = Vec<ConnectedStation>;

struct StationEntry {
    state: ConnectedStation,
    command_tx: mpsc::Sender<StationHandlerCommand>,
}

/// In-memory station state + pub/sub.
///
/// - Keeps track of all connected stations and their login status.
/// - Provides per-station command channels (for sending messages to station clients).
/// - Broadcasts state snapshots so other components can subscribe (e.g. admin live stream).
pub struct StationsHub {
    stations: RwLock<HashMap<String, StationEntry>>,
    state_tx: broadcast::Sender<HubStateEvent>,
}

/// Returned from [`StationsHub::register`]. Dropping deregisters the station.
pub struct StationRegistration {
    pub commands: mpsc::Receiver<StationHandlerCommand>,
    cleanup: Option<Box<dyn FnOnce() + Send>>,
}

impl Drop for StationRegistration {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

fn broadcast_snapshot(
    stations: &HashMap<String, StationEntry>,
    tx: &broadcast::Sender<HubStateEvent>,
) {
    let snapshot: Vec<ConnectedStation> = stations.values().map(|e| e.state.clone()).collect();
    let _ = tx.send(snapshot);
}

impl StationsHub {
    pub fn new() -> Arc<Self> {
        let (state_tx, _) = broadcast::channel(64);
        Arc::new(Self {
            stations: RwLock::new(HashMap::new()),
            state_tx,
        })
    }

    /// Subscribe to state change events (for admin live streaming).
    pub fn subscribe_state(&self) -> broadcast::Receiver<HubStateEvent> {
        self.state_tx.subscribe()
    }

    /// Snapshot of all currently connected stations.
    pub fn connected_stations(&self) -> Vec<ConnectedStation> {
        self.stations
            .read()
            .unwrap()
            .values()
            .map(|e| e.state.clone())
            .collect()
    }

    /// Register a station. Returns a registration handle with a command receiver.
    /// Errors if the IP is already registered.
    pub fn register(
        self: &Arc<Self>,
        ip: &str,
    ) -> Result<StationRegistration, crate::error::AppError> {
        let mut stations = self.stations.write().unwrap();
        if stations.contains_key(ip) {
            return Err(crate::error::AppError::FailedPrecondition(
                "station already connected".into(),
            ));
        }

        let (command_tx, command_rx) = mpsc::channel(16);
        let state = ConnectedStation {
            ip: ip.to_string(),
            connected_at: Utc::now(),
            logged_in: false,
        };

        stations.insert(ip.to_string(), StationEntry { state, command_tx });

        broadcast_snapshot(&stations, &self.state_tx);

        tracing::debug!(ip, "[HUB] registered station");

        let hub = Arc::clone(self);
        let ip_owned = ip.to_string();
        let cleanup = move || {
            let mut stations = hub.stations.write().unwrap();
            stations.remove(&ip_owned);
            broadcast_snapshot(&stations, &hub.state_tx);
            tracing::debug!(ip = %ip_owned, "[HUB] deregistered station");
        };

        Ok(StationRegistration {
            commands: command_rx,
            cleanup: Some(Box::new(cleanup)),
        })
    }

    pub fn sync_stations(&self, ips: &[&str]) {
        let stations = self.stations.read().unwrap();
        if ips.is_empty() {
            for entry in stations.values() {
                let _ = entry.command_tx.try_send(StationHandlerCommand::Sync);
            }
        } else {
            for ip in ips {
                if let Some(entry) = stations.get(*ip) {
                    let _ = entry.command_tx.try_send(StationHandlerCommand::Sync);
                }
            }
        }
    }

    /// Send a command to specific stations, or all if `ips` is empty.
    /// Non-blocking: silently drops if a station's channel is full.
    pub fn send_command(&self, command: StationCommand, ips: &[&str]) {
        let stations = self.stations.read().unwrap();
        if ips.is_empty() {
            for entry in stations.values() {
                let _ = entry.command_tx.try_send(command.clone().into());
            }
        } else {
            for ip in ips {
                if let Some(entry) = stations.get(*ip) {
                    let _ = entry.command_tx.try_send(command.clone().into());
                }
            }
        }
    }

    /// Update login status and broadcast a full snapshot.
    pub fn set_login_status(&self, ip: &str, logged_in: bool) {
        let mut stations = self.stations.write().unwrap();
        if let Some(entry) = stations.get_mut(ip) {
            entry.state.logged_in = logged_in;
            broadcast_snapshot(&stations, &self.state_tx);
        }
    }
}
