mod event_hub;
mod state;
mod types;

use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use crate::{
    domain::{
        self, BroadcastEventStream,
        event::{
            LoomEvent,
            admin::AdminEvent,
            broadcast::StationsState,
            station::{StationCommand, StationEvent},
        },
    },
    error::AppError,
    orchestrator::{
        event_hub::EventHub,
        state::MemoryStationState,
        types::{EventHub as _, StationStateStore as _},
    },
};

pub struct Orchestrator {
    hub: Arc<EventHub>,
    state: Arc<MemoryStationState>,
}

impl Orchestrator {
    pub fn new() -> Self {
        let hub = Arc::new(event_hub::EventHub::new());
        let mut state = state::MemoryStationState::new();

        // we get a new refernce to hub to move into the hook
        let hub_for_hook = hub.clone();
        state.set_on_change_hook(Box::new(move |state| {
            hub_for_hook.broadcast(state.into());
        }));

        Self {
            hub,
            state: Arc::new(state),
        }
    }
}

#[async_trait]
impl domain::Orchestrator for Orchestrator {
    fn handle_event(&self, event: domain::event::LoomEvent) {
        match event {
            LoomEvent::Station((ip, event)) => match event {
                StationEvent::LoggedIn => self.state.login(&ip),
                StationEvent::LoggedOut => {
                    let ips = [ip.as_str()];
                    self.state.logout(&ip);
                    self.sync_stations(&ips);
                }
                StationEvent::Command(command_output) => self.hub.broadcast(command_output.into()),
            },
            LoomEvent::Admin(admin_event) => match admin_event {
                AdminEvent::Station((ips, command)) => {
                    let ips: Vec<&str> = ips.iter().map(|s| s.as_str()).collect();
                    self.hub.publish_station_command(command, &ips);
                }
            },
        }
    }

    fn sync_stations(&self, ips: &[&str]) {
        self.hub
            .publish_station_command(StationCommand::SyncWallpaper, ips);
        self.hub
            .publish_station_command(StationCommand::SyncContestUrl, ips);
    }

    fn subscribe_broadcast(&self) -> BroadcastEventStream {
        let rx = self.hub.subscribe_broadcast();
        let broadcast_stream = BroadcastStream::new(rx);
        let mapped_stream = broadcast_stream.map(|e| match e {
            Ok(e) => Ok(e),
            Err(_) => Err(AppError::Internal(
                "Error reading from broadcast stream".into(),
            )),
        });

        Box::pin(mapped_stream)
    }

    async fn register_station(
        &self,
        ip: &str,
    ) -> Result<domain::StationCommandStream, crate::error::AppError> {
        let mut registration = self.hub.register_station(ip)?;
        let state_store = Arc::clone(&self.state);
        let ip_addr = ip.to_string();
        registration.add_cleanup(Box::new(move || {
            state_store.disconnect(&ip_addr);
        }));

        self.state.connect(ip);

        let stream = async_stream::stream! {
            let mut registration = registration;
            while let Some(msg) = registration.receiver.recv().await {
                yield Ok(msg);
            }
        };

        Ok(Box::pin(stream))
    }

    fn get_state(&self) -> StationsState {
        self.state.get_state()
    }
}
