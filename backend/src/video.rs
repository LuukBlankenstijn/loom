mod stream;

use std::{collections::HashMap, net::SocketAddrV4, sync::Arc};

use async_trait::async_trait;
use parking_lot::Mutex;
use tokio::sync::{mpsc, oneshot};

use crate::{
    config::VideoConfig,
    domain::{ContestRepository, Orchestrator, VideoStatus, VideoStreamer, WallpaperRepository},
    video::stream::Streamer,
};

#[derive(Default)]
struct StreamState {
    current: Option<i32>,
    streaming: Option<i32>,
    errors: HashMap<i32, String>,
}

type SharedState = Arc<Mutex<StreamState>>;

pub struct VideoMulticast {
    state: SharedState,
    address: SocketAddrV4,
    refreshes: mpsc::Sender<oneshot::Sender<()>>,
}

impl VideoMulticast {
    pub fn start(
        config: VideoConfig,
        contest_repo: Arc<dyn ContestRepository>,
        wallpaper_repo: Arc<dyn WallpaperRepository>,
        orchestrator: Arc<dyn Orchestrator>,
    ) -> Self {
        let state = SharedState::default();
        let address = config.multicast_address;
        let (refreshes, requests) = mpsc::channel(8);
        let streamer = Streamer::new(
            state.clone(),
            config,
            contest_repo,
            wallpaper_repo,
            orchestrator,
        );
        tokio::spawn(streamer.run(requests));
        Self {
            state,
            address,
            refreshes,
        }
    }
}

#[async_trait]
impl VideoStreamer for VideoMulticast {
    fn status(&self, id: i32) -> VideoStatus {
        let state = self.state.lock();
        VideoStatus {
            streaming: state.streaming == Some(id),
            error: state.errors.get(&id).cloned(),
        }
    }

    fn stream_address(&self, id: i32) -> Option<SocketAddrV4> {
        (self.state.lock().current == Some(id)).then_some(self.address)
    }

    async fn refresh(&self) {
        let (done, refreshed) = oneshot::channel();
        if self.refreshes.send(done).await.is_err() || refreshed.await.is_err() {
            tracing::error!("video streamer is not running");
        }
    }
}
