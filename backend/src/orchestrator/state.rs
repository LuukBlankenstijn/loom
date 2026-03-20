use std::{collections::HashMap, sync::RwLock};

use crate::{
    domain::event::broadcast::StationConnectionState, orchestrator::types::StateChangeHook,
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

    fn on_change(&self, update: StationConnectionState) {
        if let Some(hook) = &self.on_change {
            hook(update);
        }
    }
}

impl crate::orchestrator::types::StationStateStore for MemoryStationState {
    fn get_state(&self) -> Vec<StationConnectionState> {
        let state = self.state.read().unwrap();
        state
            .iter()
            .map(|(ip, logged_in)| StationConnectionState {
                ip: ip.clone(),
                connected: true,
                logged_in: *logged_in,
            })
            .collect()
    }

    fn connect(&self, ip: &str) {
        let mut changed = false;
        {
            let mut state = self.state.write().unwrap();
            if !state.contains_key(ip) {
                state.insert(ip.to_string(), false);
                changed = true;
            }
        }

        if changed {
            self.on_change((ip.to_string(), true, false).into());
        }
    }

    fn disconnect(&self, ip: &str) {
        let changed = {
            let mut state = self.state.write().unwrap();
            state.remove(ip).is_some()
        };

        if changed {
            self.on_change((ip.to_string(), false, false).into());
        }
    }

    fn login(&self, ip: &str) {
        let changed = {
            let mut state = self.state.write().unwrap();
            if let Some(logged_in) = state.get_mut(ip) {
                if !*logged_in {
                    *logged_in = true;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };

        if changed {
            self.on_change((ip.to_string(), true, true).into());
        }
    }

    fn logout(&self, ip: &str) {
        let changed = {
            let mut state = self.state.write().unwrap();
            if let Some(logged_in) = state.get_mut(ip) {
                if *logged_in {
                    *logged_in = false;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };

        if changed {
            self.on_change((ip.to_string(), true, false).into());
        }
    }

    fn set_on_change_hook(&mut self, hook: StateChangeHook) {
        self.on_change = Some(hook);
    }
}
