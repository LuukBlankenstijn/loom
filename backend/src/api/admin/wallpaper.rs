use derive_more::derive::Constructor;
use loom_rpc::admin::v1::{
    self as pb, create_wallpaper_request, wallpaper, wallpaper_service_server::WallpaperService,
};
use reqwest::Url;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::{
    api::admin::image,
    domain::{
        Orchestrator, VideoStreamer, Wallpaper, WallpaperKind, WallpaperListing,
        WallpaperRepository, WallpaperSource,
    },
};

const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;
pub const MAX_REQUEST_BYTES: usize = MAX_UPLOAD_BYTES + 4096;
const YOUTUBE_VIDEO_ID_LEN: usize = 11;

#[derive(Constructor)]
pub struct WallpaperHandler {
    wallpaper_repo: Arc<dyn WallpaperRepository>,
    video_streamer: Arc<dyn VideoStreamer>,
    orchestrator: Arc<dyn Orchestrator>,
}

impl WallpaperHandler {
    fn to_proto(
        &self,
        wallpaper: Wallpaper,
        is_default: bool,
        contest_ids: Vec<String>,
    ) -> pb::Wallpaper {
        let (kind, video) = match wallpaper.kind {
            WallpaperKind::Image => (pb::WallpaperKind::Image, None),
            WallpaperKind::Youtube => {
                let status = self.video_streamer.status(wallpaper.id);
                let video = pb::VideoStatus {
                    streaming: status.streaming,
                    error: status.error.unwrap_or_default(),
                };
                (pb::WallpaperKind::Youtube, Some(video))
            }
        };
        let source = match wallpaper.source {
            WallpaperSource::Stored { mime_type } => wallpaper::Source::MimeType(mime_type),
            WallpaperSource::Url(url) => wallpaper::Source::Url(url),
        };
        pb::Wallpaper {
            id: wallpaper.id,
            name: wallpaper.name,
            kind: kind.into(),
            text_color: wallpaper.text_color,
            audio: wallpaper.audio,
            source: Some(source),
            is_default,
            contest_ids,
            video,
        }
    }

    async fn sync_stations(&self) {
        self.video_streamer.refresh().await;
        self.orchestrator.sync_stations(&[]);
    }
}

fn validate_name(name: &str) -> Result<&str, Status> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Status::invalid_argument("name must not be empty"));
    }
    Ok(name)
}

fn parse_url(url: &str) -> Result<Url, Status> {
    let parsed =
        Url::parse(url).map_err(|e| Status::invalid_argument(format!("invalid url: {e}")))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(Status::invalid_argument("url must use http or https"));
    }
    Ok(parsed)
}

fn youtube_video_id(url: &Url) -> Option<String> {
    let host = url.host_str()?;
    let host = host.strip_prefix("www.").unwrap_or(host);
    let host = host.strip_prefix("m.").unwrap_or(host);
    let segments: Vec<&str> = url.path_segments()?.filter(|s| !s.is_empty()).collect();
    let id = match (host, segments.as_slice()) {
        ("youtu.be", [id]) => (*id).to_owned(),
        ("youtube.com" | "music.youtube.com", ["watch"]) => url
            .query_pairs()
            .find_map(|(key, value)| (key == "v").then(|| value.into_owned()))?,
        ("youtube.com" | "youtube-nocookie.com", ["shorts" | "embed" | "live", id]) => {
            (*id).to_owned()
        }
        _ => return None,
    };
    let valid = id.len() == YOUTUBE_VIDEO_ID_LEN
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    valid.then_some(id)
}

fn youtube_url(url: &str) -> Result<String, Status> {
    let id = youtube_video_id(&parse_url(url)?)
        .ok_or_else(|| Status::invalid_argument("url is not a link to a YouTube video"))?;
    Ok(format!("https://www.youtube.com/watch?v={id}"))
}

fn validate_color(color: &str) -> Result<&str, Status> {
    let hex = color.strip_prefix('#').unwrap_or(color);
    if !matches!(hex.len(), 3 | 6) || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(Status::invalid_argument("text color must be a hex colour"));
    }
    Ok(color)
}

fn domain_kind(kind: i32) -> Result<WallpaperKind, Status> {
    match pb::WallpaperKind::try_from(kind) {
        Ok(pb::WallpaperKind::Image) => Ok(WallpaperKind::Image),
        Ok(pb::WallpaperKind::Youtube) => Ok(WallpaperKind::Youtube),
        _ => Err(Status::invalid_argument("kind must be image or youtube")),
    }
}

#[tonic::async_trait]
impl WallpaperService for WallpaperHandler {
    async fn list_wallpapers(
        &self,
        _request: Request<()>,
    ) -> Result<Response<pb::ListWallpapersResponse>, Status> {
        let wallpapers = self
            .wallpaper_repo
            .list()
            .await?
            .into_iter()
            .map(
                |WallpaperListing {
                     wallpaper,
                     is_default,
                     contest_ids,
                 }| self.to_proto(wallpaper, is_default, contest_ids),
            )
            .collect();
        Ok(Response::new(pb::ListWallpapersResponse { wallpapers }))
    }

    async fn create_wallpaper(
        &self,
        request: Request<pb::CreateWallpaperRequest>,
    ) -> Result<Response<pb::Wallpaper>, Status> {
        let req = request.into_inner();
        let name = validate_name(&req.name)?;
        let wallpaper = match req.source {
            Some(create_wallpaper_request::Source::ImageData(data)) => {
                let mime_type = image::mime_type(&data).map_err(Status::invalid_argument)?;
                self.wallpaper_repo
                    .create_stored(name, &data, mime_type)
                    .await?
            }
            Some(create_wallpaper_request::Source::Url(source)) => {
                let kind = domain_kind(source.kind)?;
                let url = match kind {
                    WallpaperKind::Image => parse_url(&source.url)?.into(),
                    WallpaperKind::Youtube => youtube_url(&source.url)?,
                };
                self.wallpaper_repo.create_url(name, &url, kind).await?
            }
            None => return Err(Status::invalid_argument("source is required")),
        };
        Ok(Response::new(self.to_proto(wallpaper, false, Vec::new())))
    }

    async fn update_wallpaper(
        &self,
        request: Request<pb::UpdateWallpaperRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let name = req.name.as_deref().map(validate_name).transpose()?;
        let text_color = req.text_color.as_deref().map(validate_color).transpose()?;
        self.wallpaper_repo
            .update(req.id, name, text_color, req.audio)
            .await?;
        self.sync_stations().await;
        Ok(Response::new(()))
    }

    async fn delete_wallpaper(
        &self,
        request: Request<pb::DeleteWallpaperRequest>,
    ) -> Result<Response<()>, Status> {
        self.wallpaper_repo.delete(request.into_inner().id).await?;
        self.sync_stations().await;
        Ok(Response::new(()))
    }

    async fn set_default_wallpaper(
        &self,
        request: Request<pb::SetDefaultWallpaperRequest>,
    ) -> Result<Response<()>, Status> {
        self.wallpaper_repo
            .set_default(request.into_inner().wallpaper_id)
            .await?;
        self.sync_stations().await;
        Ok(Response::new(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn youtube_links_resolve_to_the_canonical_watch_url() {
        let canonical = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        for link in [
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
            "https://youtube.com/watch?list=PL123&v=dQw4w9WgXcQ&t=42s",
            "http://m.youtube.com/watch?v=dQw4w9WgXcQ",
            "https://music.youtube.com/watch?v=dQw4w9WgXcQ",
            "https://youtu.be/dQw4w9WgXcQ?si=abc",
            "https://www.youtube.com/shorts/dQw4w9WgXcQ",
            "https://www.youtube.com/embed/dQw4w9WgXcQ",
            "https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ",
            "https://www.youtube.com/live/dQw4w9WgXcQ?feature=share",
        ] {
            assert_eq!(youtube_url(link).unwrap(), canonical, "{link}");
        }
    }

    #[test]
    fn non_video_links_are_rejected() {
        for link in [
            "https://www.youtube.com/",
            "https://www.youtube.com/watch?list=PL123",
            "https://www.youtube.com/watch?v=short",
            "https://www.youtube.com/@channel",
            "https://youtu.be/",
            "https://vimeo.com/dQw4w9WgXcQ",
            "https://notyoutube.com/watch?v=dQw4w9WgXcQ",
            "ftp://youtube.com/watch?v=dQw4w9WgXcQ",
        ] {
            assert!(youtube_url(link).is_err(), "{link}");
        }
    }
}
