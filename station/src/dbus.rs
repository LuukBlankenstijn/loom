use anyhow::{Context, Result};
use greeter_dbus::GreeterServiceProxy;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tracing::debug;
use zbus::{Connection, fdo::DBusProxy, names::UniqueName};

use crate::messages::Message;

pub struct DbusClient {
    proxy: GreeterServiceProxy<'static>,
    dbus_proxy: DBusProxy<'static>,
    sender: broadcast::Sender<Message>,
    receiver: broadcast::Receiver<Message>,
}

impl DbusClient {
    pub async fn new(sender: broadcast::Sender<Message>) -> Result<Self> {
        let receiver = sender.subscribe();

        let connection = Connection::system()
            .await
            .context("failed to create system connection")?;

        let proxy = GreeterServiceProxy::new(&connection).await?;
        let dbus_proxy = zbus::fdo::DBusProxy::new(&connection).await?;

        Ok(Self {
            proxy,
            dbus_proxy,
            sender,
            receiver,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        let mut owner_changes = self
            .proxy
            .clone()
            .into_inner()
            .receive_owner_changed()
            .await?;

        loop {
            tokio::select! {
                maybe_msg = owner_changes.next() => {
                    if let Some(owner) = maybe_msg && let Err(e) = self.handle_owner_change(owner).await {
                        debug!("failed to handle owner change: {}", e);
                    };
                }

                msg = self.receiver.recv() => {
                    let msg = match msg {
                        Ok(msg) => msg,
                        Err(broadcast::error::RecvError::Closed) => anyhow::bail!("broadcast channel closed"),
                        Err(_) => continue,
                    };

                    if matches!(msg, Message::RequestLoginStatus) {
                        let message = if self.service_up().await {
                            Message::LoggedOut
                        } else {
                            Message::LoggedIn
                        };
                        let _ = self.sender.send(message);
                        continue;
                    }

                    // Check if service exists before calling
                    if self.service_up().await
                        && let Err(e) = self.handle_message(msg).await
                    {
                        debug!("Command handing failed: {}", e);
                    }
                }
            }
        }
    }

    async fn service_up(&self) -> bool {
        let name = self.proxy.inner().destination();
        match self.dbus_proxy.name_has_owner(name.to_owned()).await {
            Ok(exists) => exists,
            Err(e) => {
                tracing::trace!("Failed to check service status: {}", e);
                false
            }
        }
    }

    async fn handle_message(&self, msg: Message) -> Result<()> {
        match msg {
            Message::SetWallpaper(source) => {
                debug!("setting wallpaper source to {}", source);
                self.proxy.set_wallpaper_source(source).await?;
            }
            Message::SetContestUrl(url) => {
                debug!("setting contest url to {}", url);
                self.proxy.set_api_poller_url(url).await?;
            }
            Message::Login => {
                debug!("logging in");
                self.proxy.login().await?;
            }
            Message::LoginWithCredentials(username, password) => {
                debug!("logging in with username: {}", username);
                self.proxy
                    .login_with_credentials(username, password)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_owner_change(&self, owner: Option<UniqueName<'_>>) -> Result<()> {
        if owner.is_some() {
            debug!("greeter interface available");
            let _ = self.sender.send(Message::LoggedOut);
        } else {
            debug!("greeter interface unavailable");
            let _ = self.sender.send(Message::LoggedIn);
        }
        Ok(())
    }
}
