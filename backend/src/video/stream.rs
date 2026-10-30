use std::{
    net::{IpAddr, Ipv4Addr},
    process::{ExitStatus, Stdio},
    sync::Arc,
    time::Duration,
};

use parking_lot::Mutex;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, ChildStderr, Command},
    sync::{mpsc, oneshot},
    time::{Instant, MissedTickBehavior},
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
const FIRST_RETRY_DELAY: Duration = Duration::from_secs(5);
const MAX_RETRY_DELAY: Duration = Duration::from_secs(300);
const HEALTHY_RUN: Duration = Duration::from_secs(60);
const RESOLVE_TIMEOUT: Duration = Duration::from_secs(30);
const VIDEO_FORMAT: &str = "bv*[height<=1080][protocol=https]/b[height<=1080][protocol=https]/bv*[protocol=https]/b[protocol=https]";
const VIDEO_AND_AUDIO_FORMAT: &str = "bv*[height<=1080][protocol=https]+ba[protocol=https]/b[height<=1080][protocol=https]/bv*[protocol=https]+ba[protocol=https]/b[protocol=https]";
const VIDEO_FILTER: &str = "scale=1920:1080:force_original_aspect_ratio=increase,crop=1920:1080,setsar=1,fps=30,format=yuv420p";

type ErrorLine = Arc<Mutex<Option<String>>>;

struct Wanted {
    id: i32,
    url: String,
    audio: bool,
}

struct Stream {
    id: i32,
    audio: bool,
    process: Child,
    started: Instant,
    last_error: ErrorLine,
}

struct Backoff {
    id: i32,
    failures: u32,
    retry_at: Option<Instant>,
}

pub(super) struct Streamer {
    state: SharedState,
    config: VideoConfig,
    contest_repo: Arc<dyn ContestRepository>,
    wallpaper_repo: Arc<dyn WallpaperRepository>,
    orchestrator: Arc<dyn Orchestrator>,
    stream: Option<Stream>,
    backoff: Option<Backoff>,
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
            backoff: None,
        }
    }

    pub async fn run(mut self, mut requests: mpsc::Receiver<oneshot::Sender<()>>) {
        let mut announced = None;
        let mut recheck = tokio::time::interval(RECHECK_INTERVAL);
        recheck.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            let retry_at = self.backoff.as_ref().and_then(|backoff| backoff.retry_at);
            let requested = tokio::select! {
                Some(done) = requests.recv() => Some(done),
                _ = recheck.tick() => None,
                _ = sleep_until(retry_at) => {
                    if let Some(backoff) = &mut self.backoff {
                        backoff.retry_at = None;
                    }
                    None
                }
                status = exited(&mut self.stream) => {
                    self.handle_exit(status);
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

    fn handle_exit(&mut self, status: std::io::Result<ExitStatus>) {
        let Some(stream) = self.set_stream(None) else {
            return;
        };
        let reason = stream.last_error.lock().take();
        let error = match (status, reason) {
            (_, Some(reason)) => format!("ffmpeg stopped streaming: {reason}"),
            (Ok(status), None) => format!("ffmpeg stopped streaming ({status})"),
            (Err(e), None) => format!("failed to wait for ffmpeg: {e}"),
        };
        tracing::warn!(id = stream.id, %error, "video stream stopped");
        self.state.lock().errors.insert(stream.id, error);
        self.fail(stream.id, stream.started.elapsed() >= HEALTHY_RUN);
    }

    fn fail(&mut self, id: i32, after_healthy_run: bool) {
        let failures = match &self.backoff {
            Some(backoff) if backoff.id == id && !after_healthy_run => backoff.failures + 1,
            _ => 1,
        };
        let delay = FIRST_RETRY_DELAY
            .saturating_mul(2u32.saturating_pow(failures - 1))
            .min(MAX_RETRY_DELAY);
        tracing::info!(id, ?delay, "retrying video stream later");
        self.backoff = Some(Backoff {
            id,
            failures,
            retry_at: Some(Instant::now() + delay),
        });
    }

    async fn reconcile(&mut self) {
        let wanted = match self.wanted().await {
            Ok(wanted) => wanted,
            Err(e) => {
                tracing::warn!(error = %e, "failed to resolve the current wallpaper");
                return;
            }
        };
        let wanted_id = wanted.as_ref().map(|wanted| wanted.id);
        self.state.lock().current = wanted_id;

        if let Some(stream) = &self.stream
            && Some((stream.id, stream.audio)) == wanted.as_ref().map(|w| (w.id, w.audio))
        {
            if stream.started.elapsed() >= HEALTHY_RUN {
                self.state.lock().errors.remove(&stream.id);
                self.backoff = None;
            }
            return;
        }

        if let Some(mut stream) = self.set_stream(None) {
            let _ = stream.process.kill().await;
            tracing::info!(id = stream.id, "stopped streaming video");
        }
        let Some(Wanted { id, url, audio }) = wanted else {
            return;
        };
        if let Some(backoff) = &self.backoff
            && backoff.id == id
            && backoff.retry_at.is_some()
        {
            return;
        }

        let started = match resolve_sources(&url, audio).await {
            Ok(sources) => self.spawn(&sources, audio),
            Err(error) => Err(error),
        };
        match started {
            Ok((process, last_error)) => {
                tracing::info!(id, %url, audio, address = %self.config.multicast_address, "streaming video");
                self.set_stream(Some(Stream {
                    id,
                    audio,
                    process,
                    started: Instant::now(),
                    last_error,
                }));
            }
            Err(error) => {
                tracing::warn!(id, %url, %error, "failed to stream video");
                self.state.lock().errors.insert(id, error);
                self.fail(id, false);
            }
        }
    }

    async fn wanted(&self) -> Result<Option<Wanted>, AppError> {
        let wallpaper = resolve_wallpaper(&*self.contest_repo, &*self.wallpaper_repo, None).await?;
        Ok(
            wallpaper.and_then(|wallpaper| match (wallpaper.kind, wallpaper.source) {
                (WallpaperKind::Youtube, WallpaperSource::Url(url)) => Some(Wanted {
                    id: wallpaper.id,
                    url,
                    audio: wallpaper.audio,
                }),
                _ => None,
            }),
        )
    }

    fn set_stream(&mut self, stream: Option<Stream>) -> Option<Stream> {
        self.state.lock().streaming = stream.as_ref().map(|stream| stream.id);
        std::mem::replace(&mut self.stream, stream)
    }

    fn spawn(&self, sources: &[String], audio: bool) -> Result<(Child, ErrorLine), String> {
        let mut destination = format!(
            "udp://{}?ttl={}&pkt_size=1316&bitrate=8000000",
            self.config.multicast_address, self.config.multicast_ttl
        );
        if let Some(interface) = &self.config.multicast_interface {
            let address = interface_address(interface)?;
            destination.push_str(&format!("&localaddr={address}"));
        }

        let mut command = Command::new("ffmpeg");
        command.args(["-nostdin", "-hide_banner", "-loglevel", "error"]);
        for source in sources {
            command.args(["-re", "-stream_loop", "-1", "-i", source]);
        }
        command.args(["-map", "0:v:0", "-sn", "-dn", "-vf", VIDEO_FILTER]);
        if audio {
            let audio_input = format!("{}:a:0", sources.len() - 1);
            command
                .args(["-map", &audio_input])
                .args(["-c:a", "aac", "-b:a", "128k", "-ac", "2", "-ar", "48000"]);
        } else {
            command.arg("-an");
        }
        let mut process = command
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
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("failed to run ffmpeg: {e}"))?;

        let last_error = Arc::new(Mutex::new(None));
        if let Some(stderr) = process.stderr.take() {
            tokio::spawn(collect_errors(stderr, last_error.clone()));
        }
        Ok((process, last_error))
    }
}

async fn collect_errors(stderr: ChildStderr, last_error: ErrorLine) {
    let mut lines = BufReader::new(stderr).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim();
        if !line.is_empty() {
            tracing::warn!(target: "loom_backend::video::ffmpeg", "{line}");
            *last_error.lock() = Some(line.to_owned());
        }
    }
}

async fn resolve_sources(url: &str, audio: bool) -> Result<Vec<String>, String> {
    let format = if audio {
        VIDEO_AND_AUDIO_FORMAT
    } else {
        VIDEO_FORMAT
    };
    let output = Command::new("yt-dlp")
        .args(["--no-playlist", "--no-cache-dir", "--get-url"])
        .args(["--format", format, "--format-sort", "hdr:SDR", "--", url])
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
    let sources: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_owned)
        .collect();
    if sources.is_empty() {
        return Err("yt-dlp returned no video URL".to_owned());
    }
    Ok(sources)
}

async fn sleep_until(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(deadline).await,
        None => std::future::pending().await,
    }
}

async fn exited(stream: &mut Option<Stream>) -> std::io::Result<ExitStatus> {
    match stream {
        Some(stream) => stream.process.wait().await,
        None => std::future::pending().await,
    }
}

fn interface_address(name: &str) -> Result<Ipv4Addr, String> {
    local_ip_address::list_afinet_netifas()
        .map_err(|e| format!("failed to list network interfaces: {e}"))?
        .into_iter()
        .find_map(|(interface, address)| match address {
            IpAddr::V4(address) if interface == name => Some(address),
            _ => None,
        })
        .ok_or_else(|| format!("multicast interface {name} has no IPv4 address"))
}
