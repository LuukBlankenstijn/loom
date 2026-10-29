use std::{process::Stdio, sync::Arc, time::Duration};

use tokio::{
    process::{Child, Command},
    sync::{mpsc, oneshot},
};

use crate::{
    config::VideoConfig,
    domain::{
        ContestRepository, Orchestrator, WallpaperKind, WallpaperRepository, WallpaperSource,
        resolve_wallpaper,
    },
    error::AppError,
    video::SharedState,
};

const RECHECK_INTERVAL: Duration = Duration::from_secs(30);
const RESTART_DELAY: Duration = Duration::from_secs(5);
const RESOLVE_TIMEOUT: Duration = Duration::from_secs(30);
const SOURCE_FORMAT: &str = "bv*[height<=1080][protocol=https]/b[height<=1080][protocol=https]/bv*[protocol=https]/b[protocol=https]";
const VIDEO_FILTER: &str = "scale=1920:1080:force_original_aspect_ratio=increase,crop=1920:1080,setsar=1,fps=30,format=yuv420p";

struct Stream {
    id: i32,
    process: Child,
}

pub(super) struct Streamer {
    state: SharedState,
    config: VideoConfig,
    contest_repo: Arc<dyn ContestRepository>,
    wallpaper_repo: Arc<dyn WallpaperRepository>,
    orchestrator: Arc<dyn Orchestrator>,
    stream: Option<Stream>,
}

impl Streamer {
    pub fn new(
        state: SharedState,
        config: VideoConfig,
        contest_repo: Arc<dyn ContestRepository>,
        wallpaper_repo: Arc<dyn WallpaperRepository>,
        orchestrator: Arc<dyn Orchestrator>,
    ) -> Self {
        Self {
            state,
            config,
            contest_repo,
            wallpaper_repo,
            orchestrator,
            stream: None,
        }
    }

    pub async fn run(mut self, mut requests: mpsc::Receiver<oneshot::Sender<()>>) {
        let mut announced = None;
        let mut recheck = tokio::time::interval(RECHECK_INTERVAL);
        loop {
            let requested = tokio::select! {
                Some(done) = requests.recv() => Some(done),
                _ = recheck.tick() => None,
                (id, status) = exited(&mut self.stream) => {
                    let error = match status {
                        Ok(status) => format!("ffmpeg stopped streaming ({status})"),
                        Err(e) => format!("failed to wait for ffmpeg: {e}"),
                    };
                    tracing::warn!(id, %error, "video stream stopped, restarting");
                    self.set_stream(None);
                    self.state.lock().errors.insert(id, error);
                    tokio::time::sleep(RESTART_DELAY).await;
                    None
                }
            };

            self.reconcile().await;
            let current = self.state.lock().current;
            match requested {
                Some(done) => {
                    let _ = done.send(());
                }
                None if current != announced => self.orchestrator.sync_stations(&[]),
                None => {}
            }
            announced = current;
        }
    }

    async fn reconcile(&mut self) {
        let wanted = match self.wanted().await {
            Ok(wanted) => wanted,
            Err(e) => {
                tracing::warn!(error = %e, "failed to resolve the current wallpaper");
                return;
            }
        };
        let wanted_id = wanted.as_ref().map(|(id, _)| *id);
        self.state.lock().current = wanted_id;
        if self.stream.as_ref().map(|stream| stream.id) == wanted_id {
            return;
        }

        if let Some(mut stream) = self.set_stream(None) {
            let _ = stream.process.kill().await;
            tracing::info!(id = stream.id, "stopped streaming video");
        }
        let Some((id, url)) = wanted else {
            return;
        };
        match self.start(&url).await {
            Ok(process) => {
                tracing::info!(id, %url, address = %self.config.multicast_address, "streaming video");
                self.state.lock().errors.remove(&id);
                self.set_stream(Some(Stream { id, process }));
            }
            Err(error) => {
                tracing::warn!(id, %url, %error, "failed to stream video");
                self.state.lock().errors.insert(id, error);
            }
        }
    }

    async fn wanted(&self) -> Result<Option<(i32, String)>, AppError> {
        let wallpaper = resolve_wallpaper(&*self.contest_repo, &*self.wallpaper_repo, None).await?;
        Ok(
            wallpaper.and_then(|wallpaper| match (wallpaper.kind, wallpaper.source) {
                (WallpaperKind::Youtube, WallpaperSource::Url(url)) => Some((wallpaper.id, url)),
                _ => None,
            }),
        )
    }

    fn set_stream(&mut self, stream: Option<Stream>) -> Option<Stream> {
        self.state.lock().streaming = stream.as_ref().map(|stream| stream.id);
        std::mem::replace(&mut self.stream, stream)
    }

    async fn start(&self, url: &str) -> Result<Child, String> {
        let source = resolve_source(url).await?;

        let mut destination = format!(
            "udp://{}?ttl={}&pkt_size=1316&bitrate=8000000",
            self.config.multicast_address, self.config.multicast_ttl
        );
        if let Some(interface) = self.config.multicast_interface {
            destination.push_str(&format!("&localaddr={interface}"));
        }

        Command::new("ffmpeg")
            .args(["-nostdin", "-hide_banner", "-loglevel", "error"])
            .args(["-re", "-stream_loop", "-1", "-i", &source])
            .args(["-map", "0:v:0", "-an", "-sn", "-dn", "-vf", VIDEO_FILTER])
            .args(["-c:v", "libx264", "-preset", "veryfast", "-crf", "23"])
            .args([
                "-maxrate",
                "5M",
                "-bufsize",
                "2M",
                "-g",
                "30",
                "-sc_threshold",
                "0",
            ])
            .args(["-f", "mpegts", &destination])
            .stdin(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("failed to run ffmpeg: {e}"))
    }
}

async fn resolve_source(url: &str) -> Result<String, String> {
    let output = Command::new("yt-dlp")
        .args(["--no-playlist", "--no-cache-dir", "--get-url"])
        .args([
            "--format",
            SOURCE_FORMAT,
            "--format-sort",
            "hdr:SDR",
            "--",
            url,
        ])
        .stdin(Stdio::null())
        .kill_on_drop(true)
        .output();
    let output = tokio::time::timeout(RESOLVE_TIMEOUT, output)
        .await
        .map_err(|_| format!("yt-dlp did not resolve the video within {RESOLVE_TIMEOUT:?}"))?
        .map_err(|e| format!("failed to run yt-dlp: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let reason = stderr
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("no error output");
        return Err(format!("yt-dlp failed ({}): {reason}", output.status));
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(str::to_owned)
        .ok_or_else(|| "yt-dlp returned no video URL".to_owned())
}

async fn exited(stream: &mut Option<Stream>) -> (i32, std::io::Result<std::process::ExitStatus>) {
    match stream {
        Some(stream) => (stream.id, stream.process.wait().await),
        None => std::future::pending().await,
    }
}
