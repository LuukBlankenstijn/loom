use contest_greeter_dbus::{GreeterService, GreeterServiceBackend};
use iced::Subscription;
use iced::futures::channel::mpsc::Sender;
use iced::futures::{self, SinkExt};
use iced::stream;
use log::{error, info};
use zbus::conn::Builder;
#[derive(Clone, Debug)]
pub enum DbusMessage {
    SetWallpaper(String),
    Login,
    SetApiUrl(String),
}

pub fn dbus_service_subscription() -> Subscription<DbusMessage> {
    Subscription::run(|| {
        stream::channel(16, |output: Sender<DbusMessage>| async move {
            let greeter_service = GreeterService::new(GreeterDbusBackend {
                sender: output.clone(),
            });

            let result = Builder::system()
                .and_then(|b| b.name("nl.luukblankenstijn.ContestGreeterService"))
                .and_then(|b| {
                    b.serve_at(
                        "/nl/luukblankenstijn/ContestGreeterService",
                        greeter_service,
                    )
                });

            match result {
                Ok(builder) => match builder.build().await {
                    Ok(_connection) => {
                        info!(
                            "[DBus-Service] Service started: nl.luukblankenstijn.ContestGreeterService"
                        );
                        std::future::pending::<()>().await;
                    }
                    Err(e) => {
                        error!("[DBus-Service] Failed to build D-Bus connection: {}", e);
                    }
                },
                Err(e) => {
                    error!("[DBus-Service] configuration failed: {}", e);
                }
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
}
