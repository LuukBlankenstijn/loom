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
        Orchestrator, Wallpaper, WallpaperKind, WallpaperListing, WallpaperRepository,
        WallpaperSource,
    },
};

const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;
pub const MAX_REQUEST_BYTES: usize = MAX_UPLOAD_BYTES + 4096;

#[derive(Constructor)]
pub struct WallpaperHandler {
    wallpaper_repo: Arc<dyn WallpaperRepository>,
    orchestrator: Arc<dyn Orchestrator>,
}

fn to_proto(wallpaper: Wallpaper, is_default: bool, contest_ids: Vec<String>) -> pb::Wallpaper {
    let kind = match wallpaper.kind {
        WallpaperKind::Image => pb::WallpaperKind::Image,
        WallpaperKind::Video => pb::WallpaperKind::Video,
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
        source: Some(source),
        is_default,
        contest_ids,
    }
}

fn validate_name(name: &str) -> Result<&str, Status> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Status::invalid_argument("name must not be empty"));
    }
    Ok(name)
}

fn validate_url(url: &str) -> Result<&str, Status> {
    let parsed = Url::parse(url).map_err(|e| Status::invalid_argument(format!("invalid url: {e}")))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(Status::invalid_argument("url must use http or https"));
    }
    Ok(url)
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
        Ok(pb::WallpaperKind::Video) => Ok(WallpaperKind::Video),
        _ => Err(Status::invalid_argument("kind must be image or video")),
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
                 }| to_proto(wallpaper, is_default, contest_ids),
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
                let url = validate_url(&source.url)?;
                let kind = domain_kind(source.kind)?;
                self.wallpaper_repo.create_url(name, url, kind).await?
            }
            None => return Err(Status::invalid_argument("source is required")),
        };
        Ok(Response::new(to_proto(wallpaper, false, Vec::new())))
    }

    async fn update_wallpaper(
        &self,
        request: Request<pb::UpdateWallpaperRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let name = req.name.as_deref().map(validate_name).transpose()?;
        let text_color = req.text_color.as_deref().map(validate_color).transpose()?;
        self.wallpaper_repo.update(req.id, name, text_color).await?;
        self.orchestrator.sync_stations(&[]);
        Ok(Response::new(()))
    }

    async fn delete_wallpaper(
        &self,
        request: Request<pb::DeleteWallpaperRequest>,
    ) -> Result<Response<()>, Status> {
        self.wallpaper_repo.delete(request.into_inner().id).await?;
        self.orchestrator.sync_stations(&[]);
        Ok(Response::new(()))
    }

    async fn set_default_wallpaper(
        &self,
        request: Request<pb::SetDefaultWallpaperRequest>,
    ) -> Result<Response<()>, Status> {
        self.wallpaper_repo
            .set_default(request.into_inner().wallpaper_id)
            .await?;
        self.orchestrator.sync_stations(&[]);
        Ok(Response::new(()))
    }
}
