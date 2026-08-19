use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use base64::{Engine, engine::general_purpose::STANDARD};
use bytes::{Buf, BufMut, Bytes};
use http::uri::PathAndQuery;
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{State, ipc::Channel};
use tokio::sync::Mutex as AsyncMutex;
use tokio_stream::StreamExt;
use tonic::{
    Request, Status,
    codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder},
    metadata::{Ascii, MetadataValue},
    transport::{Channel as Transport, ClientTlsConfig},
};

#[derive(Serialize)]
pub struct GrpcFailure {
    code: i32,
    message: String,
}

impl From<Status> for GrpcFailure {
    fn from(status: Status) -> Self {
        Self {
            code: status.code() as i32,
            message: status.message().to_string(),
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum StreamEvent {
    Message { data: String },
    End,
    Error { code: i32, message: String },
}

#[derive(Clone, Copy)]
struct RawCodec;

impl Codec for RawCodec {
    type Encode = Vec<u8>;
    type Decode = Bytes;
    type Encoder = RawEncoder;
    type Decoder = RawDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        RawEncoder
    }

    fn decoder(&mut self) -> Self::Decoder {
        RawDecoder
    }
}

struct RawEncoder;

impl Encoder for RawEncoder {
    type Item = Vec<u8>;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, dst: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        dst.put_slice(&item);
        Ok(())
    }
}

struct RawDecoder;

impl Decoder for RawDecoder {
    type Item = Bytes;
    type Error = Status;

    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let remaining = src.remaining();
        Ok(Some(src.copy_to_bytes(remaining)))
    }
}

pub struct Backend {
    server: String,
    token: Option<MetadataValue<Ascii>>,
    transport: AsyncMutex<Option<Transport>>,
    streams: Arc<Mutex<HashMap<u64, tokio::task::AbortHandle>>>,
    next_stream_id: AtomicU64,
}

impl Backend {
    pub fn new(server: String, auth: Option<String>) -> anyhow::Result<Self> {
        let token = auth
            .map(|token| MetadataValue::try_from(format!("Bearer {token}")))
            .transpose()?;
        Ok(Self {
            server,
            token,
            transport: AsyncMutex::new(None),
            streams: Arc::new(Mutex::new(HashMap::new())),
            next_stream_id: AtomicU64::new(0),
        })
    }

    async fn transport(&self) -> Result<Transport, Status> {
        let mut current = self.transport.lock().await;
        if let Some(transport) = current.as_ref() {
            return Ok(transport.clone());
        }

        let mut endpoint = Transport::from_shared(self.server.clone())
            .map_err(|e| Status::invalid_argument(e.to_string()))?
            .connect_timeout(Duration::from_secs(5))
            .http2_keep_alive_interval(Duration::from_secs(15))
            .keep_alive_timeout(Duration::from_secs(10));

        if self.server.starts_with("https") {
            endpoint = endpoint
                .tls_config(ClientTlsConfig::new().with_native_roots())
                .map_err(|e| Status::internal(e.to_string()))?;
        }

        let transport = endpoint
            .connect()
            .await
            .map_err(|e| Status::unavailable(e.to_string()))?;
        *current = Some(transport.clone());
        Ok(transport)
    }

    fn request(&self, body: Vec<u8>) -> Request<Vec<u8>> {
        let mut request = Request::new(body);
        if let Some(token) = self.token.as_ref() {
            request
                .metadata_mut()
                .insert("authorization", token.clone());
        }
        request
    }
}

fn decode_request(request: &str) -> Result<Vec<u8>, Status> {
    STANDARD
        .decode(request)
        .map_err(|e| Status::invalid_argument(e.to_string()))
}

fn decode_path(path: String) -> Result<PathAndQuery, Status> {
    PathAndQuery::try_from(path).map_err(|e| Status::invalid_argument(e.to_string()))
}

async fn client(backend: &Backend) -> Result<tonic::client::Grpc<Transport>, Status> {
    let mut grpc = tonic::client::Grpc::new(backend.transport().await?);
    grpc.ready()
        .await
        .map_err(|e| Status::unavailable(e.to_string()))?;
    Ok(grpc)
}

#[tauri::command]
pub async fn grpc_unary(
    backend: State<'_, Backend>,
    path: String,
    request: String,
) -> Result<String, GrpcFailure> {
    let body = decode_request(&request)?;
    let path = decode_path(path)?;
    let response = client(&backend)
        .await?
        .unary(backend.request(body), path, RawCodec)
        .await?;
    Ok(STANDARD.encode(response.into_inner()))
}

#[tauri::command]
pub async fn grpc_server_stream(
    backend: State<'_, Backend>,
    path: String,
    request: String,
    on_event: Channel<StreamEvent>,
) -> Result<u64, GrpcFailure> {
    let body = decode_request(&request)?;
    let path = decode_path(path)?;
    let request = backend.request(body);
    let mut grpc = client(&backend).await?;

    let id = backend.next_stream_id.fetch_add(1, Ordering::Relaxed);
    let streams = Arc::clone(&backend.streams);

    let task = tokio::spawn(async move {
        let outcome: Result<(), Status> = async {
            let mut stream = grpc
                .server_streaming(request, path, RawCodec)
                .await?
                .into_inner();
            while let Some(message) = stream.next().await.transpose()? {
                if on_event
                    .send(StreamEvent::Message {
                        data: STANDARD.encode(message),
                    })
                    .is_err()
                {
                    return Ok(());
                }
            }
            Ok(())
        }
        .await;

        let _ = match outcome {
            Ok(()) => on_event.send(StreamEvent::End),
            Err(status) => on_event.send(StreamEvent::Error {
                code: status.code() as i32,
                message: status.message().to_string(),
            }),
        };
        streams.lock().remove(&id);
    });

    backend.streams.lock().insert(id, task.abort_handle());
    Ok(id)
}

#[tauri::command]
pub fn grpc_cancel_stream(backend: State<'_, Backend>, stream_id: u64) {
    if let Some(task) = backend.streams.lock().remove(&stream_id) {
        task.abort();
    }
}
