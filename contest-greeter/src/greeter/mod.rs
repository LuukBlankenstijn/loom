mod lightdm;

use anyhow::{Result, anyhow};
use lightdm_contest_rs_greeter::CoreName;
use log::{error, info, warn};
use serde::Deserialize;
use tokio::sync::mpsc;
use types::{GreeterMessage, SystemBus, UiMessage};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct GreeterConfig {
    session: Option<String>,
}

pub struct Greeter {
    greeter: lightdm::Greeter,
    conf: GreeterConfig,
}

impl Greeter {
    pub fn new(conf: GreeterConfig) -> Result<Self> {
        let greeter = match lightdm::Greeter::new() {
            Ok(greeter) => {
                if let Err(e) = greeter.connect_to_daemon() {
                    return Err(anyhow!(
                        "[Greeter] Failed to connect to LightDM daemon: {e}"
                    ));
                }

                greeter
            }
            Err(e) => {
                return Err(anyhow!("[Greeter] failed to construct greeter: {e}"));
            }
        };
        Ok(Self { greeter, conf })
    }

    pub async fn run(&self, bus: impl SystemBus) {
        let message_bus = bus.clone();
        self.greeter
            .set_message_handler(move |message, message_type| {
                match message_type {
                    lightdm::MessageType::Info => info!("[Greeter] {}", message),
                    lightdm::MessageType::Error => error!("[Greeter] {}", message),
                }
                if let lightdm::MessageType::Error = message_type {
                    message_bus.send_to(CoreName::UI, UiMessage::SetError(message.to_string()));
                }
            });

        let auth_bus = bus.clone();
        let session = self.conf.session.clone();
        self.greeter
            .set_authentication_complete_handler(move |success| {
                if success {
                    auth_bus.send_to(
                        CoreName::Greeter,
                        GreeterMessage::StartSession(session.clone()),
                    );
                    info!("[Greeter] authentication succeeded");
                } else {
                    auth_bus.send_to(
                        CoreName::UI,
                        UiMessage::SetError("Authentication failed".to_string()),
                    );
                    warn!("[Greeter] authentication failed");
                }
            });

        let (tx, mut rx) = mpsc::channel(16);
        bus.register(CoreName::Greeter, tx);

        info!("[Greeter] starting greeter loop");
        while let Some(msg) = rx.recv().await {
            match msg {
                GreeterMessage::Login(username, password) => self.authenticate(username, password),
                GreeterMessage::StartSession(session_option) => self.start_session(session_option),
            }
        }
    }

    fn authenticate(&self, username: String, password: String) {
        self.greeter.respond_to_secret_prompts(password);

        match self.greeter.authenticate(&username) {
            Ok(_) => {}
            Err(e) => {
                warn!("[Greeter] failed to authenticate: {}", e);
            }
        };
    }

    pub fn start_session(&self, session: Option<String>) {
        match self.greeter.start_session(session.as_deref()) {
            Ok(_) => {}
            Err(e) => {
                error!("[Greeter] failed to start session: {}", e)
            }
        }
    }
}
