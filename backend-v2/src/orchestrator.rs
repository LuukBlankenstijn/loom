mod event_hub;
mod state;
mod types;

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::broadcast::Receiver;

use crate::{
    domain::{
        self,
        event::{
            LoomEvent,
            admin::AdminEvent,
            broadcast::BroadcastEvent,
            station::{StationCommand, StationEvent},
        },
    },
    orchestrator::{
        event_hub::EventHub,
        state::MemoryStationState,
        types::{EventHub as _, StationStateStore as _},
    },
};

pub struct Orchestrator {
    hub: Arc<EventHub>,
    state: Arc<MemoryStationState>,

    contest_url: Option<String>,
}

impl Orchestrator {
    pub fn new(contest_url: Option<String>) -> Self {
        let hub = Arc::new(event_hub::EventHub::new());
        let mut state = state::MemoryStationState::new();

        // we get a new refernce to hub to move into the hook
        let hub_for_hook = hub.clone();
        state.set_on_change_hook(Box::new(move |state| {
            hub_for_hook.broadcast(state.into());
        }));

        Self {
            hub,
            contest_url,
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
                StationEvent::LoggedOut => self.state.login(&ip),
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

    fn sync_wallpaper(&self, ips: &[&str]) {
        self.hub
            .publish_station_command(StationCommand::SyncWallpaper, ips);
    }

    fn sync_api_url(&self, ips: &[&str], contest_id: String) {
        if let Some(base_url) = self.contest_url.clone() {
            let contest_url = format!("{base_url}/api/v4/contests/{}", contest_id);
            self.hub
                .publish_station_command(StationCommand::SetContestUrl(contest_url), ips);
        }
    }

    fn subscribe_broadcast(&self) -> Receiver<BroadcastEvent> {
        self.hub.subscribe_broadcast()
    }

    async fn register_station(
        &self,
        ip: &str,
    ) -> Result<domain::EventStream, crate::error::AppError> {
        let registration = self.hub.register_station(ip)?;
        self.state.connect(ip);

        let state_store = Arc::clone(&self.state);
        let ip_addr = ip.to_string();

        let stream = async_stream::stream! {
            let mut registration = registration;
            while let Some(msg) = registration.receiver.recv().await {
                yield Ok(msg);
            }

            state_store.disconnect(&ip_addr);
        };

        Ok(Box::pin(stream))
    }
}
