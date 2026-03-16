mod convert;

use std::sync::Arc;

use derive_more::derive::Constructor;
use loom_rpc::admin::v1::{self as pb, team_service_server::TeamService};
use tonic::{Request, Response, Status};

use crate::domain::{ContestRepository, Orchestrator, TeamRepository};

#[derive(Constructor)]
pub struct TeamHandler {
    contest_repo: Arc<dyn ContestRepository>,
    team_repo: Arc<dyn TeamRepository>,
    orchestrator: Arc<dyn Orchestrator>,
}

#[tonic::async_trait]
impl TeamService for TeamHandler {
    async fn get_active_teams(
        &self,
        _request: Request<()>,
    ) -> Result<Response<pb::TeamsResponse>, Status> {
        let teams = match self.contest_repo.get_next_contest().await? {
            Some(contest) => self.team_repo.get_all(&contest.id).await?,
            None => vec![],
        };
        Ok(Response::new(pb::TeamsResponse {
            teams: teams.into_iter().map(Into::into).collect(),
        }))
    }

    async fn set_ip(&self, request: Request<pb::SetIpRequest>) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let maybe_ip = self
            .team_repo
            .set_ip(&req.team_id, req.ip.as_deref())
            .await?;
        if let Some(ip) = maybe_ip {
            self.orchestrator.sync_stations(&[&ip]);
        }
        Ok(Response::new(()))
    }
}
