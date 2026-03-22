mod event_hub;
mod state;
mod types;

use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use loom_core::event::{
    LoomEvent,
    admin::{AdminCommand, AdminEvent, CommandOutput},
    broadcast::{BroadcastEvent, StationConnectionState},
    station::{StationCommand, StationEvent},
};
use tokio_stream::wrappers::BroadcastStream;

use crate::{
    domain::{self, BroadcastEventStream},
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
            hub_for_hook.broadcast(vec![state].into());
        }));

        Self {
            hub,
            state: Arc::new(state),
        }
    }
}

#[async_trait]
impl domain::Orchestrator for Orchestrator {
    fn handle_event(&self, event: LoomEvent) {
        match event {
            LoomEvent::Station((ip, event)) => match event {
                StationEvent::LoggedIn => self.state.login(&ip),
                StationEvent::LoggedOut => {
                    let ips = [ip.as_str()];
                    self.state.logout(&ip);
                    self.sync_stations(&ips);
                }
                StationEvent::CustomCommand(command_output) => {
                    let ids = [command_output.admin_id.as_str()];
                    self.hub.publish_admin_command(
                        CommandOutput {
                            id: command_output.id,
                            output: command_output.output,
                        }
                        .into(),
                        &ids,
                    );
                }
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
    ) -> Result<domain::CommandStream<StationCommand>, crate::error::AppError> {
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

    async fn register_admin(
        &self,
        id: &str,
    ) -> Result<domain::CommandStream<AdminCommand>, crate::error::AppError> {
        let mut registration = self.hub.register_admin(id)?;
        let state_store = Arc::clone(&self.state);
        let id_clone = id.to_string();
        registration.add_cleanup(Box::new(move || {
            state_store.disconnect(&id_clone);
        }));

        let stream = async_stream::stream! {
            let mut registration = registration;
            while let Some(msg) = registration.receiver.recv().await {
                yield Ok(msg);
            }
        };

        Ok(Box::pin(stream))
    }

    fn get_state(&self) -> Vec<StationConnectionState> {
        self.state.get_state()
    }

    fn broadcast(&self, event: BroadcastEvent) {
        self.hub.broadcast(event);
    }
}
