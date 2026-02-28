use greeter_dbus::{GreeterService, GreeterServiceBackend, GreeterServiceProxy};
use iced::Subscription;
use iced::futures::channel::mpsc::Sender;
use iced::futures::{self, SinkExt};
use iced::stream;
use log::{error, info};
use zbus::conn::Builder;
use zbus::proxy::Defaults;
#[derive(Clone, Debug)]
pub enum DbusMessage {
    SetWallpaper(String),
    Login,
    LoginWithCredentials(String, String),
    SetApiUrl(String),
}

pub fn dbus_service_subscription() -> Subscription<DbusMessage> {
    Subscription::run(|| {
        stream::channel(16, |output: Sender<DbusMessage>| async move {
            let greeter_service = GreeterService::new(GreeterDbusBackend {
                sender: output.clone(),
            });

            let name = match GreeterServiceProxy::DESTINATION.as_ref() {
                Some(name) => name.as_str(),
                None => {
                    error!("[DBus-Service] Missing destination name in proxy definition");
                    return;
                }
            };

            let path = match GreeterServiceProxy::PATH.as_ref() {
                Some(path) => path,
                None => {
                    error!("[DBus-Service] Missing path in proxy definition");
                    return;
                }
            };

            let connection_result = Builder::system()
                .map_err(|e| format!("System bus connection failed: {}", e))
                .and_then(|b| b.name(name).map_err(|e| format!("Invalid name: {}", e)))
                .and_then(|b| {
                    b.serve_at(path, greeter_service)
                        .map_err(|e| format!("Path error: {}", e))
                });

            match connection_result {
                Ok(builder) => match builder.build().await {
                    Ok(_connection) => {
                        info!("[DBus-Service] Service started: {}", name);
                        std::future::pending::<()>().await;
                    }
                    Err(e) => error!("[DBus-Service] Failed to build D-Bus connection: {}", e),
                },
                Err(e) => error!("[DBus-Service] Configuration failed: {}", e),
            }
        })
    })
}

struct GreeterDbusBackend {
    sender: futures::channel::mpsc::Sender<DbusMessage>,
}

impl GreeterServiceBackend for GreeterDbusBackend {
    fn set_wallpaper_source(&self, url: String) {
        let mut sender = self.sender.clone();
        tokio::spawn(async move {
            let _ = sender.send(DbusMessage::SetWallpaper(url)).await;
        });
    }

    fn set_api_poller_url(&self, url: String) {
        let mut sender = self.sender.clone();
        tokio::spawn(async move {
            let _ = sender.send(DbusMessage::SetApiUrl(url)).await;
        });
    }

    fn login(&self) {
        let mut sender = self.sender.clone();
        tokio::spawn(async move {
            let _ = sender.send(DbusMessage::Login).await;
        });
    }

    fn login_with_credentials(&self, username: String, password: String) {
        let mut sender = self.sender.clone();
        tokio::spawn(async move {
            let _ = sender
                .send(DbusMessage::LoginWithCredentials(username, password))
                .await;
        });
    }
}
