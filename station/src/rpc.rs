use std::time::Duration;

use anyhow::Result;
use loom_rpc::{
    command::v1::CustomCommandOutput,
    stations::v1::{
        ClientMessage, ServerMessage, client_message, server_message,
        station_service_client::StationServiceClient,
    },
};
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio_stream::StreamExt;
use tonic::Streaming;
use tracing::{error, info, warn};

use crate::messages::{RpcCommand, RpcEvent};

impl From<RpcCommand> for ClientMessage {
    fn from(value: RpcCommand) -> Self {
        let message = match value {
            RpcCommand::LoggedIn => client_message::Message::LoggedIn(()),
            RpcCommand::LoggedOut => client_message::Message::LoggedOut(()),
            RpcCommand::CustomCommandOutput(id, output) => {
                client_message::Message::CommandOutput(CustomCommandOutput { id, output })
            }
        };
        Self {
            message: Some(message),
        }
    }
}

impl TryFrom<ServerMessage> for RpcEvent {
    type Error = anyhow::Error;

    fn try_from(value: ServerMessage) -> std::result::Result<Self, Self::Error> {
        let message = match value.message {
            Some(message) => match message {
                server_message::Message::SetWallpaperSource(source) => {
                    RpcEvent::SetWallpaper(source)
                }
                server_message::Message::SetContestUrl(url) => RpcEvent::SetContestUrl(url),
                server_message::Message::Login(_) => RpcEvent::Login,
                server_message::Message::Logout(_) => RpcEvent::Logout,
                server_message::Message::LoginWithCredentials(login_with_credentials_message) => {
                    RpcEvent::LoginWithCredentials(
                        login_with_credentials_message.username,
                        login_with_credentials_message.password,
                    )
                }
                server_message::Message::CustomCommand(custom_command_message) => {
                    RpcEvent::CustomCommand(
                        custom_command_message.id,
                        custom_command_message.command,
                    )
                }
            },
            None => return Err(anyhow::anyhow!("Message is none")),
        };
        Ok(message)
    }
}

pub struct RpcClient {
    address: String,
    command_receiver: Receiver<RpcCommand>,
    event_sender: Sender<RpcEvent>,
}

impl RpcClient {
    pub async fn new(address: String) -> (Self, Sender<RpcCommand>, Receiver<RpcEvent>) {
        let (command_tx, command_rx) = channel(32);
        let (event_tx, event_rx) = channel(32);

        let client = Self {
            address,
            command_receiver: command_rx,
            event_sender: event_tx,
        };
        (client, command_tx, event_rx)
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            match self.connect_and_subscribe().await {
                Ok((mut rpc_tx, mut server_stream)) => {
                    info!("Connected to rpc server at {}", self.address);
                    let _ = self.event_sender.send(RpcEvent::RequestLoginStatus).await;
                    if let Err(e) = self.process_io(&mut rpc_tx, &mut server_stream).await {
                        warn!("Connection lost: {}. Reconnecting...", e);
                    }
                }
                Err(e) => {
                    error!("failed to connect to {}: {}", self.address, e);
                }
            }

            // drop everything while not connected
            while self.command_receiver.try_recv().is_ok() {}

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn connect_and_subscribe(
        &self,
    ) -> Result<(Sender<ClientMessage>, Streaming<ServerMessage>)> {
        let transport = tonic::transport::Channel::from_shared(self.address.clone())?
            .connect_timeout(Duration::from_secs(5))
            .connect()
            .await?;
        let mut client = StationServiceClient::new(transport);

        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let stream = tokio_stream::wrappers::ReceiverStream::new(rx);

        let response = client.subscribe(stream).await?;
        Ok((tx, response.into_inner()))
    }

    async fn process_io(
        &mut self,
        rpc_tx: &mut Sender<ClientMessage>,
        server_stream: &mut Streaming<ServerMessage>,
    ) -> Result<()> {
        tokio::pin!(server_stream);

        loop {
            tokio::select! {
                Some(cmd) = self.command_receiver.recv() => {
                    if let Err(e) = rpc_tx.send(cmd.into()).await {
                        return Err(anyhow::anyhow!("gRPC pipe broken: {}", e));
                    }
                }

                maybe_msg = server_stream.next() => {
                    match maybe_msg {
                        Some(Ok(msg)) => {
                            if let Ok(msg) = msg.try_into() {
                                let _ = self.event_sender.send(msg).await;
                            }
                        }
                        Some(Err(e)) => return Err(anyhow::anyhow!("Stream error: {}", e)),
                        None => return Err(anyhow::anyhow!("Stream closed by server")),
                    }
                }
            }
        }
    }
}
