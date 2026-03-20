use tokio::sync::mpsc;
use tonic::transport::Channel;
use tonic::Streaming;

use loom_rpc::stations::v1::{
    station_service_client::StationServiceClient, ClientMessage, ServerMessage,
    client_message::Message as ClientMsg,
};

/// Manages the bidirectional gRPC stream between this station and the backend.
pub struct StationClient {
    /// Send client messages (logged_in, logged_out, command_output) to the backend.
    pub sender: mpsc::Sender<ClientMessage>,
    /// Receive server commands (sync_wallpaper, login, logout, etc.) from the backend.
    pub receiver: Streaming<ServerMessage>,
}

impl StationClient {
    /// Connect to the backend and start the bidirectional station stream.
    pub async fn connect(endpoint: &str) -> Result<Self, tonic::Status> {
        let channel = Channel::from_shared(endpoint.to_string())
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .connect()
            .await
            .map_err(|e| tonic::Status::unavailable(e.to_string()))?;

        let mut client = StationServiceClient::new(channel);

        let (tx, mut rx) = mpsc::channel::<ClientMessage>(16);

        // Wrap the receiver into a tonic-compatible stream
        let outbound = async_stream::stream! {
            while let Some(msg) = rx.recv().await {
                yield msg;
            }
        };

        let response = client.subscribe(outbound).await?;
        let receiver = response.into_inner();

        Ok(Self { sender: tx, receiver })
    }

    /// Notify the backend that this station is now logged in.
    pub async fn send_logged_in(&self) -> Result<(), mpsc::error::SendError<ClientMessage>> {
        self.sender
            .send(ClientMessage {
                message: Some(ClientMsg::LoggedIn(())),
            })
            .await
    }

    /// Notify the backend that this station is now logged out.
    pub async fn send_logged_out(&self) -> Result<(), mpsc::error::SendError<ClientMessage>> {
        self.sender
            .send(ClientMessage {
                message: Some(ClientMsg::LoggedOut(())),
            })
            .await
    }
}
