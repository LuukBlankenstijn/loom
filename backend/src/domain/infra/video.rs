use std::net::SocketAddrV4;

use async_trait::async_trait;

#[derive(Debug, Clone, Default)]
pub struct VideoStatus {
    pub streaming: bool,
    pub error: Option<String>,
}

#[async_trait]
pub trait VideoStreamer: Send + Sync {
    fn status(&self, id: i32) -> VideoStatus;
    fn stream_address(&self, id: i32) -> Option<SocketAddrV4>;
    async fn refresh(&self);
}
