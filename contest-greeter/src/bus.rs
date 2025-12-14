use std::collections::HashMap;

use lightdm_contest_rs_greeter::{CoreName, ServiceChannel, SystemMsg};
use log::{debug, error, info};
use tokio::sync::mpsc;

pub async fn start_bus(mut rx: mpsc::Receiver<SystemMsg>) {
    let mut registry: HashMap<CoreName, Box<dyn ServiceChannel>> = HashMap::new();

    info!("[Bus] starting message loop");
    while let Some(msg) = rx.recv().await {
        match msg {
            SystemMsg::Register { name, channel } => {
                if registry.contains_key(&name) {
                    error!("[Bus] service {:?} already registered", name)
                }
                debug!("[Bus] registered {:?}", name);
                registry.insert(name, channel);
            }
            SystemMsg::Route { to, payload } => {
                if let Some(service) = registry.get(&to) {
                    service.send_any(payload);
                } else {
                    debug!("[Bus] service {:?} not found for incoming message", to)
                }
            }
        }
    }
}
