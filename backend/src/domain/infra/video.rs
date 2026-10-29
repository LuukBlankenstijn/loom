use std::{net::SocketAddrV4, sync::Arc};

use async_trait::async_trait;

use crate::domain::Orchestrator;

#[derive(Debug, Clone, Default)]
pub struct VideoStatus {
    pub streaming: bool,
    pub error: Option<String>,
}

#[async_trait]
pub trait VideoStreamer: Send + Sync {
    fn status(&self, id: i32) -> VideoStatus;
    fn stream_address(&self, id: i32) -> Option<SocketAddrV4>;
    fn forget(&self, id: i32);
    async fn refresh(&self);
}

pub fn resync_wallpapers(
    video_streamer: Arc<dyn VideoStreamer>,
    orchestrator: Arc<dyn Orchestrator>,
) {
    tokio::spawn(async move {
        video_streamer.refresh().await;
        orchestrator.sync_stations(&[]);
    });
}
