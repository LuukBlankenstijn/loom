use anyhow::{Context, Result};
use greeter_dbus::GreeterServiceProxy;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio_stream::StreamExt;
use tracing::{debug, trace};
use zbus::{Connection, fdo::DBusProxy, names::UniqueName};

use crate::messages::{DbusCommand, DbusEvent};

pub struct DbusClient {
    proxy: GreeterServiceProxy<'static>,
    dbus_proxy: DBusProxy<'static>,
    receiver: Receiver<DbusCommand>,
    sender: Sender<DbusEvent>,
}

impl DbusClient {
    pub async fn new() -> Result<(Self, Sender<DbusCommand>, Receiver<DbusEvent>)> {
        let (cmd_tx, cmd_rx) = channel::<DbusCommand>(32);
        let (event_tx, event_rx) = channel::<DbusEvent>(32);

        let connection = Connection::system()
            .await
            .context("failed to create system connection")?;

        let proxy = GreeterServiceProxy::new(&connection).await?;
        let dbus_proxy = zbus::fdo::DBusProxy::new(&connection).await?;

        let client = Self {
            proxy,
            dbus_proxy,
            receiver: cmd_rx,
            sender: event_tx,
        };

        Ok((client, cmd_tx, event_rx))
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
                    let cmd = msg.ok_or_else(|| anyhow::anyhow!("command receiver closed"))?;

                    // Check if service exists before calling
                    if self.service_up().await {
                        if let Err(e) = self.handle_command(cmd).await {
                            debug!("Command handing failed: {}", e);
                        }
                    } else {
                        trace!("Dropped command {:#?}: service {} not online", cmd, self.proxy.inner().destination());
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

    async fn handle_command(&self, cmd: DbusCommand) -> Result<()> {
        match cmd {
            DbusCommand::SetWallpaper(source) => {
                debug!("setting wallpaper source to {}", source);
                self.proxy.set_wallpaper_source(source).await?;
            }
            DbusCommand::SetContestUrl(url) => {
                debug!("setting contest url to {}", url);
                self.proxy.set_api_poller_url(url).await?;
            }
            DbusCommand::Login => {
                debug!("logging in");
                self.proxy.login().await?;
            }
            DbusCommand::LoginWithCredentials(username, password) => {
                debug!("logging in with username: {}", username);
                self.proxy
                    .login_with_credentials(username, password)
                    .await?;
            }
            DbusCommand::GetLoginStatus => {
                let message = if self.service_up().await {
                    DbusEvent::LoggedIn
                } else {
                    DbusEvent::LoggedOut
                };
                self.sender.send(message).await?;
            }
        }
        Ok(())
    }

    async fn handle_owner_change(&self, owner: Option<UniqueName<'_>>) -> Result<()> {
        if owner.is_some() {
            debug!("greeter interface available");
            self.sender.send(DbusEvent::LoggedOut).await?;
        } else {
            debug!("greeter interface unavailable");
            self.sender.send(DbusEvent::LoggedIn).await?;
        }
        Ok(())
    }
}
