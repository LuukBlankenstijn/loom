use chrono::{Local, TimeZone};
use contest_greeter_dbus::{GreeterService, GreeterServiceBackend};
use log::{error, info};
use types::{GreeterMessage, SystemSender, UiMessage};
use zbus::conn::Builder;

struct GreeterDbusBackend<T: SystemSender> {
    bus: T,
}

impl<T: SystemSender + Sync> GreeterServiceBackend for GreeterDbusBackend<T> {
    fn set_wallpaper_source(&self, url: String) {
        self.bus
            .send_to(types::CoreName::UI, UiMessage::SetWallpaper(Some(url)));
    }

    fn set_countdown_endtime(&self, end_time: i64) -> zbus::fdo::Result<()> {
        let datetime = match Local.timestamp_millis_opt(end_time) {
            chrono::offset::LocalResult::Single(result) => result,
            chrono::offset::LocalResult::Ambiguous(_, _) => {
                return Err(zbus::fdo::Error::InvalidArgs(
                    "Ambiguous timezone".to_string(),
                ));
            }
            chrono::offset::LocalResult::None => {
                return Err(zbus::fdo::Error::InvalidArgs(
                    "Error converting to timezone".to_string(),
                ));
            }
        };
        self.bus.send_to(
            types::CoreName::UI,
            UiMessage::SetCountdownEndtime {
                end_time: Some(datetime),
            },
        );
        Ok(())
    }

    fn disable_countdown(&self) {
        self.bus.send_to(
            types::CoreName::UI,
            UiMessage::SetCountdownEndtime { end_time: None },
        );
    }

    fn login(&self) {
        self.bus
            .send_to(types::CoreName::Greeter, GreeterMessage::Login());
    }
}

pub async fn run_dbus_service(bus: impl SystemSender) {
    let greeter_service = GreeterService::new(GreeterDbusBackend { bus });
    let result = Builder::system()
        .and_then(|b| b.name("nl.luukblankenstijn.ContestGreeterService"))
        .and_then(|b| {
            b.serve_at(
                "/nl/luukblankenstijn/ContestGreeterService",
                greeter_service,
            )
        });

    match result {
        Ok(builder) => {
            // Now we await the async build process
            match builder.build().await {
                Ok(_connection) => {
                    info!(
                        "[DBus-Service] Service started: nl.luukblankenstijn.ContestGreeterService"
                    );
                    std::future::pending::<()>().await;
                }
                Err(e) => {
                    error!("[DBus-Service] Failed to build D-Bus connection: {}", e);
                }
            }
        }
        Err(e) => {
            error!("[DBus-Service] configuration failed: {}", e);
        }
    }
}
