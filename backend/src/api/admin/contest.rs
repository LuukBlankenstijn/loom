use derive_more::derive::Constructor;
use loom_proto_bridge::IntoProto;
use loom_rpc::admin::v1::{self as pb, contest_service_server::ContestService};
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::domain::{ContestRepository, MapRepository, Orchestrator};

#[derive(Constructor)]
pub struct ContestHandler {
    contest_repo: Arc<dyn ContestRepository>,
    map_repo: Arc<dyn MapRepository>,
    orchestrator: Arc<dyn Orchestrator>,
}

#[tonic::async_trait]
impl ContestService for ContestHandler {
    async fn get_next_contest(
        &self,
        _request: Request<()>,
    ) -> Result<Response<pb::Contest>, Status> {
        let contest = self
            .contest_repo
            .get_next_contest()
            .await?
            .ok_or_else(|| Status::not_found("no upcoming contest"))?;

        let map_id = self
            .map_repo
            .get_by_contest(&contest.id)
            .await
            .ok()
            .flatten()
            .map(|m| m.id);

        let mut pb_contest: pb::Contest = contest.into_proto();
        pb_contest.map_id = map_id;
        Ok(Response::new(pb_contest))
    }

    async fn set_wallpaper(
        &self,
        request: Request<pb::UploadWallpaperRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        match req.image_data.filter(|d| !d.is_empty()) {
            Some(data) => {
                let mime_type = validate_image(&data).map_err(Status::invalid_argument)?;
                self.contest_repo
                    .set_wallpaper(&req.contest_id, &data, mime_type)
                    .await?;
            }
            None => {
                self.contest_repo.delete_wallpaper(&req.contest_id).await?;
            }
        }
        self.orchestrator.sync_stations(&[]);
        Ok(Response::new(()))
    }

    async fn set_wallpaper_text_color(
        &self,
        request: Request<pb::SetWallpaperTextColorRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        self.contest_repo
            .set_wallpaper_text_color(&req.contest_id, &req.color)
            .await?;
        // update stations
        self.orchestrator.sync_stations(&[]);
        Ok(Response::new(()))
    }

    async fn set_map(&self, request: Request<pb::SetMapRequest>) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        self.contest_repo
            .set_map(&req.contest_id, req.map_id)
            .await?;
        Ok(Response::new(()))
    }
}

fn validate_image(data: &[u8]) -> Result<&'static str, String> {
    use image::ImageFormat;
    use std::io::Cursor;

    let format = image::guess_format(data).map_err(|_| "unsupported image format".to_string())?;

    let reader = image::ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| format!("failed to read image: {e}"))?;

    reader
        .into_dimensions()
        .map_err(|e| format!("invalid image data: {e}"))?;

    let mime = match format {
        ImageFormat::Png => "image/png",
        ImageFormat::Jpeg => "image/jpeg",
        ImageFormat::Gif => "image/gif",
        ImageFormat::WebP => "image/webp",
        ImageFormat::Bmp => "image/bmp",
        ImageFormat::Tiff => "image/tiff",
        _ => return Err("unsupported image format".to_string()),
    };

    Ok(mime)
}
