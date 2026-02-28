use std::time::Duration;

use anyhow::Result;
use loom_rpc::{
    command::v1::CustomCommandOutput,
    stations::v1::{
        ClientMessage, ServerMessage, client_message, server_message,
        station_service_client::StationServiceClient,
    },
};
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tonic::Streaming;
use tracing::{error, info, warn};

use crate::messages::Message;

impl TryFrom<Message> for ClientMessage {
    type Error = ();

    fn try_from(value: Message) -> std::result::Result<Self, ()> {
        let message = match value {
            Message::LoggedIn => client_message::Message::LoggedIn(()),
            Message::LoggedOut => client_message::Message::LoggedOut(()),
            Message::CommandOutput { id, output } => {
                client_message::Message::CommandOutput(CustomCommandOutput { id, output })
            }
            _ => return Err(()),
        };
        Ok(Self {
            message: Some(message),
        })
    }
}

impl TryFrom<ServerMessage> for Message {
    type Error = anyhow::Error;

    fn try_from(value: ServerMessage) -> std::result::Result<Self, Self::Error> {
        let message = match value.message {
            Some(message) => match message {
                server_message::Message::SetWallpaperSource(source) => {
                    Message::SetWallpaper(source)
                }
                server_message::Message::SetContestUrl(url) => Message::SetContestUrl(url),
                server_message::Message::Login(_) => Message::Login,
                server_message::Message::Logout(_) => Message::Logout,
                server_message::Message::LoginWithCredentials(msg) => {
                    Message::LoginWithCredentials(msg.username, msg.password)
                }
                server_message::Message::CustomCommand(msg) => Message::RunCommand {
                    id: msg.id,
                    command: msg.command,
                },
            },
            None => return Err(anyhow::anyhow!("Message is none")),
        };
        Ok(message)
    }
}

pub struct RpcClient {
    address: String,
    sender: broadcast::Sender<Message>,
    receiver: broadcast::Receiver<Message>,
}

impl RpcClient {
    pub fn new(address: String, sender: broadcast::Sender<Message>) -> Self {
        let receiver = sender.subscribe();
        Self {
            address,
            sender,
            receiver,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            match self.connect_and_subscribe().await {
                Ok((mut rpc_tx, mut server_stream)) => {
                    info!("Connected to rpc server at {}", self.address);
                    let _ = self.sender.send(Message::RequestLoginStatus);
                    if let Err(e) = self.process_io(&mut rpc_tx, &mut server_stream).await {
                        warn!("Connection lost: {}. Reconnecting...", e);
                    }
                }
                Err(e) => {
                    error!("failed to connect to {}: {}", self.address, e);
                }
            }

            // drain stale messages while disconnected
            while self.receiver.try_recv().is_ok() {}

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn connect_and_subscribe(
        &self,
    ) -> Result<(
        tokio::sync::mpsc::Sender<ClientMessage>,
        Streaming<ServerMessage>,
    )> {
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
        rpc_tx: &mut tokio::sync::mpsc::Sender<ClientMessage>,
        server_stream: &mut Streaming<ServerMessage>,
    ) -> Result<()> {
        tokio::pin!(server_stream);

        loop {
            tokio::select! {
                msg = self.receiver.recv() => {
                    let msg = match msg {
                        Ok(msg) => msg,
                        Err(broadcast::error::RecvError::Closed) => anyhow::bail!("broadcast channel closed"),
                        Err(_) => continue,
                    };
                    if let Ok(client_msg) = ClientMessage::try_from(msg) && let Err(e) = rpc_tx.send(client_msg).await {
                        return Err(anyhow::anyhow!("gRPC pipe broken: {}", e));
                    }
                }

                maybe_msg = server_stream.next() => {
                    match maybe_msg {
                        Some(Ok(msg)) => {
                            if let Ok(msg) = Message::try_from(msg) {
                                let _ = self.sender.send(msg);
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
