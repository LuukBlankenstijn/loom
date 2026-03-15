use std::{collections::HashMap, sync::RwLock};

use crate::{
    domain::event::broadcast::{StationConnectionState, StationsState},
    orchestrator::types::{StateChangeHook, StationStateStore},
};

pub struct MemoryStationState {
    state: RwLock<HashMap<String, bool>>,
    on_change: Option<StateChangeHook>,
}

impl MemoryStationState {
    pub fn new() -> Self {
        Self {
            state: Default::default(),
            on_change: Default::default(),
        }
    }

    fn on_change(&self) {
        let state = self.get_state();
        if let Some(hook) = &self.on_change {
            hook(state);
        }
    }
}

impl crate::orchestrator::types::StationStateStore for MemoryStationState {
    fn get_state(&self) -> StationsState {
        let state = self.state.read().unwrap();
        let state = state
            .iter()
            .map(|(ip, logged_in)| StationConnectionState {
                ip: ip.clone(),
                logged_in: *logged_in,
            })
            .collect();
        StationsState(state)
    }

    fn connect(&self, ip: &str) {
        let mut state = self.state.write().unwrap();
        if !state.contains_key(ip) {
            state.insert(ip.to_string(), false);
            self.on_change();
        }
    }

    fn disconnect(&self, ip: &str) {
        let mut state = self.state.write().unwrap();
        let result = state.remove(ip);
        // there was previous state, notify
        if result.is_some() {
            self.on_change();
        }
    }

    fn login(&self, ip: &str) {
        let mut state = self.state.write().unwrap();
        if state.contains_key(ip) {
            state.insert(ip.to_string(), true);
            self.on_change();
        }
    }

    fn logout(&self, ip: &str) {
        let mut state = self.state.write().unwrap();
        if state.contains_key(ip) {
            state.insert(ip.to_string(), false);
            self.on_change();
        }
    }

    fn set_on_change_hook(&mut self, hook: StateChangeHook) {
        self.on_change = Some(hook);
    }
}
