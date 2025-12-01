use anyhow::{Result, anyhow};
use log::{debug, error, info};
use tokio::sync::mpsc;

use crate::lightdm;

#[derive(Debug)]
pub enum CoreCommand {
    StartSession(Option<String>),
    SetError(String),
}

pub struct Greeter {
    greeter: lightdm::Greeter,
}

impl Greeter {
    pub fn new() -> Result<Self> {
        let greeter = match lightdm::Greeter::new() {
            Ok(greeter) => {
                if let Err(e) = greeter.connect_to_daemon() {
                    return Err(anyhow!("Failed to connect to LightDM daemon: {e}"));
                }

                greeter
            }
            Err(e) => {
                return Err(anyhow!("failed to construct greeter: {e}"));
            }
        };
        Ok(Self { greeter })
    }

    pub fn init(&self, core_tx: mpsc::UnboundedSender<CoreCommand>) {
        let message_tx = core_tx.clone();
        self.greeter
            .set_message_handler(move |message, message_type| {
                match message_type {
                    lightdm::MessageType::Info => info!("Greeter: {}", message),
                    lightdm::MessageType::Error => error!("Greeter error: {}", message),
                }
                if let lightdm::MessageType::Error = message_type {
                    let _ = message_tx.send(CoreCommand::SetError(message.to_string()));
                }
            });

        let auth_tx = core_tx.clone();

        self.greeter
            .set_authentication_complete_handler(move |success| {
                if success {
                    let _ = auth_tx.send(CoreCommand::StartSession(None));
                    info!("Authentication completed successfully");
                } else {
                    let _ =
                        auth_tx.send(CoreCommand::SetError("Authentication failed".to_string()));
                    info!("Authentication failed");
                }
            });
    }

    pub fn authenticate(&self, username: String, password: String) {
        self.greeter.respond_to_secret_prompts(password);

        match self.greeter.authenticate(&username) {
            Ok(_) => {}
            Err(e) => {
                debug!("Failed to authenticate: {}", e);
            }
        };
    }

    pub fn start_session(&self, session: Option<&str>) {
        match self.greeter.start_session(session) {
            Ok(_) => {}
            Err(e) => {
                debug!("Failed to start session: {}", e)
            }
        }
    }
}
