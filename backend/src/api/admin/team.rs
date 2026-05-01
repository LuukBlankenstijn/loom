use std::sync::Arc;

use derive_more::derive::Constructor;
use loom_core::event::broadcast::StationAssignment;
use loom_proto_bridge::IntoProto;
use loom_rpc::admin::v1::{self as pb, team_service_server::TeamService};
use tonic::{Request, Response, Status};

use crate::domain::{ContestRepository, MapRepository, Orchestrator, TeamRepository};

#[derive(Constructor)]
pub struct TeamHandler {
    contest_repo: Arc<dyn ContestRepository>,
    team_repo: Arc<dyn TeamRepository>,
    map_repo: Arc<dyn MapRepository>,
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
            teams: teams.into_iter().map(IntoProto::into_proto).collect(),
        }))
    }

    async fn set_ip(&self, request: Request<pb::SetIpRequest>) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let change = self
            .team_repo
            .set_ip(&req.team_id, req.ip.as_deref())
            .await?;

        let mut sync_ips: Vec<&str> = Vec::new();
        if let Some(ip) = change.old.as_deref() {
            sync_ips.push(ip);
        }
        if let Some(ip) = change.new.as_deref()
            && !sync_ips.contains(&ip)
        {
            sync_ips.push(ip);
        }
        if !sync_ips.is_empty() {
            self.orchestrator.sync_stations(&sync_ips);
        }

        let mut assignments: Vec<StationAssignment> = Vec::new();
        for ip in &sync_ips {
            if let Some(seat_id) = self.map_repo.get_seat_id_by_ip(ip).await? {
                let team_name = self.team_repo.get_by_ip(ip).await?.map(|t| t.name);
                assignments.push(StationAssignment {
                    seat_id,
                    station_ip: Some((*ip).to_string()),
                    team_name,
                });
            }
        }
        if !assignments.is_empty() {
            self.orchestrator.broadcast(assignments.into());
        }

        Ok(Response::new(()))
    }
}
