use std::time::Duration;

use anyhow::{Context, Result};
use local_ip_address::linux::local_ip;
use loom_rpc::station::v1::{
    CustomCommandOutput, StationCommand, StationEvent, station_command, station_event,
    station_service_client::StationServiceClient,
};
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tonic::{
    Request, Status, Streaming,
    metadata::{Ascii, MetadataValue},
    service::Interceptor,
    transport::ClientTlsConfig,
};
use tracing::{error, info, warn};

use crate::messages::Message;

#[derive(Clone)]
pub struct AuthInterceptor {
    token: Option<MetadataValue<Ascii>>,
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        if let Some(token) = self.token.as_ref() {
            request
                .metadata_mut()
                .insert("authorization", token.clone());
        }
        Ok(request)
    }
}

impl TryFrom<Message> for StationEvent {
    type Error = ();

    fn try_from(value: Message) -> std::result::Result<Self, ()> {
        let message = match value {
            Message::LoggedIn => station_event::Message::LoggedIn(()),
            Message::LoggedOut => station_event::Message::LoggedOut(()),
            Message::CommandOutput {
                id,
                output,
                admin_id,
            } => station_event::Message::CommandOutput(CustomCommandOutput {
                id,
                output,
                admin_id,
            }),
            _ => return Err(()),
        };
        Ok(Self {
            message: Some(message),
        })
    }
}

impl TryFrom<StationCommand> for Message {
    type Error = anyhow::Error;

    fn try_from(value: StationCommand) -> std::result::Result<Self, Self::Error> {
        let message = match value.message {
            Some(message) => match message {
                station_command::Message::SyncWallpaper(()) => Message::SyncWallpaper,
                station_command::Message::SyncContestUrl(()) => Message::SetContestUrl,
                station_command::Message::Login(_) => Message::Login,
                station_command::Message::Logout(_) => Message::Logout,
                station_command::Message::LoginWithCredentials(msg) => {
                    Message::LoginWithCredentials(msg.username, msg.password)
                }
                station_command::Message::CustomCommand(msg) => Message::RunCommand {
                    id: msg.id,
                    command: msg.command,
                    admin_id: msg.admin_id,
                },
            },
            None => return Err(anyhow::anyhow!("Message is none")),
        };
        Ok(message)
    }
}

pub struct RpcClient {
    address: String,
    auth_token: Option<String>,
    sender: broadcast::Sender<Message>,
    receiver: broadcast::Receiver<Message>,
}

impl RpcClient {
    pub fn new(
        address: String,
        auth_token: Option<String>,
        sender: broadcast::Sender<Message>,
    ) -> Self {
        let receiver = sender.subscribe();
        Self {
            address,
            auth_token,
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
                    error!("failed to connect to {}: {:?}", self.address, e);
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
        tokio::sync::mpsc::Sender<StationEvent>,
        Streaming<StationCommand>,
    )> {
        let mut endpoint = tonic::transport::Channel::from_shared(self.address.clone())?
            .connect_timeout(Duration::from_secs(5))
            .http2_keep_alive_interval(Duration::from_secs(15))
            .keep_alive_timeout(Duration::from_secs(10))
            .keep_alive_while_idle(true);

        if self.address.starts_with("https") {
            endpoint = endpoint.tls_config(ClientTlsConfig::new().with_native_roots())?;
        }
        let transport = endpoint.connect().await?;

        let token = self
            .auth_token
            .as_ref()
            .map(|s| format!("Bearer {}", s).parse::<MetadataValue<Ascii>>())
            .transpose()
            .map_err(|e| Status::invalid_argument(format!("Invalid metadata: {}", e)))?;

        let interceptor = AuthInterceptor { token };

        let mut client = StationServiceClient::with_interceptor(transport, interceptor);

        // create streams
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let stream = tokio_stream::wrappers::ReceiverStream::new(rx);

        // add ip to meta data
        let mut request = Request::new(stream);

        let ip = local_ip().context("Could not get local IP")?;
        let header_value = ip
            .to_string()
            .parse()
            .context("Failed to parse local IP into a valid header value")?;
        request
            .metadata_mut()
            .insert("x-loom-station-ip", header_value);

        let response = client.subscribe(request).await?;
        Ok((tx, response.into_inner()))
    }

    async fn process_io(
        &mut self,
        rpc_tx: &mut tokio::sync::mpsc::Sender<StationEvent>,
        server_stream: &mut Streaming<StationCommand>,
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
                    if let Ok(client_msg) = StationEvent::try_from(msg) && let Err(e) = rpc_tx.send(client_msg).await {
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
