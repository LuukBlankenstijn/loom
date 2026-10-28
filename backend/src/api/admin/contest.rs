use derive_more::derive::Constructor;
use loom_proto_bridge::IntoProto;
use loom_rpc::admin::v1::{self as pb, contest_service_server::ContestService};
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::domain::{ContestRepository, MapRepository, Orchestrator, WallpaperRepository};

#[derive(Constructor)]
pub struct ContestHandler {
    contest_repo: Arc<dyn ContestRepository>,
    map_repo: Arc<dyn MapRepository>,
    wallpaper_repo: Arc<dyn WallpaperRepository>,
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

        let wallpaper_id = self
            .wallpaper_repo
            .get_for_contest(&contest.id)
            .await?
            .map(|w| w.id);

        let mut pb_contest: pb::Contest = contest.into_proto();
        pb_contest.map_id = map_id;
        pb_contest.wallpaper_id = wallpaper_id;
        Ok(Response::new(pb_contest))
    }

    async fn set_wallpaper(
        &self,
        request: Request<pb::SetWallpaperRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        self.wallpaper_repo
            .assign_contest(&req.contest_id, req.wallpaper_id)
            .await?;
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
