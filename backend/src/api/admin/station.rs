use derive_more::derive::Constructor;
use futures::{Stream, future::try_join_all};
use loom_core::event::admin::AdminEvent;
use loom_proto_bridge::{IntoProto, TryIntoCore};
use loom_rpc::admin::v1::{
    self as pb, AssignTeamRequest, CommandOutputRequest, DeleteStationRequest,
    station_service_server::StationService,
};
use std::{pin::Pin, sync::Arc};
use tokio_stream::StreamExt as _;
use tonic::{Request, Response, Status};

use crate::domain::{ContestRepository, Orchestrator, StationRepository, TeamRepository};

#[derive(Constructor)]
pub struct StationHandler {
    contest_repo: Arc<dyn ContestRepository>,
    station_repo: Arc<dyn StationRepository>,
    team_repo: Arc<dyn TeamRepository>,
    orchestrator: Arc<dyn Orchestrator>,
}

#[tonic::async_trait]
impl StationService for StationHandler {
    async fn get_stations(
        &self,
        _request: Request<()>,
    ) -> Result<Response<pb::StationsResponse>, Status> {
        let stations = self.station_repo.get_all().await?;
        Ok(Response::new(pb::StationsResponse {
            stations: stations.into_iter().map(IntoProto::into_proto).collect(),
        }))
    }

    async fn delete_station(
        &self,
        request: Request<DeleteStationRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let station = self.station_repo.get(&req.ip).await?;
        if let Ok(Some(team)) = self.team_repo.get_by_ip(&station.ip).await {
            self.team_repo.set_ip(&team.id, None).await?;
        }
        self.station_repo.delete(&req.ip).await?;

        Ok(Response::new(()))
    }

    async fn assign_team(
        &self,
        request: Request<AssignTeamRequest>,
    ) -> Result<Response<()>, Status> {
        let ips = request.into_inner().ips;

        let (contest_opt, all_stations) = tokio::try_join!(
            self.contest_repo.get_next_contest(),
            self.station_repo.get_all(),
        )
        .map_err(|e| Status::internal(format!("Dependency error: {}", e)))?;

        let contest = match contest_opt {
            Some(c) => c,
            None => return Ok(Response::new(())),
        };

        let mut teams: Vec<_> = self
            .team_repo
            .get_all(&contest.id)
            .await?
            .into_iter()
            .filter(|t| t.ip.is_none())
            .collect();

        let mut stations: Vec<_> = all_stations
            .into_iter()
            .filter(|s| ips.contains(&s.ip))
            .collect();

        let mut update_futures = Vec::new();
        let mut updated_ips = Vec::new();
        let pair_count = std::cmp::min(stations.len(), teams.len());

        for _ in 0..pair_count {
            if let (Some(station), Some(team)) = (stations.pop(), teams.pop()) {
                let team_id = team.id.clone();
                let station_ip = station.ip.clone();
                updated_ips.push(station_ip.clone());

                let fut = async move { self.team_repo.set_ip(&team_id, Some(&station_ip)).await };
                update_futures.push(Box::pin(fut));
            }
        }

        if !update_futures.is_empty() {
            try_join_all(update_futures)
                .await
                .map_err(|e| Status::internal(format!("Failed to batch update teams: {}", e)))?;
        }
        let sync_refs: Vec<&str> = updated_ips.iter().map(|s| s.as_str()).collect();
        self.orchestrator.sync_stations(&sync_refs);

        Ok(Response::new(()))
    }

    async fn send_command(&self, request: Request<pb::AdminEvent>) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let event: AdminEvent = req.try_into_core()?;
        self.orchestrator.handle_event(event.into());
        Ok(().into())
    }

    type CommandOutputStream =
        Pin<Box<dyn Stream<Item = Result<pb::CustomCommandOutput, Status>> + Send>>;

    async fn command_output(
        &self,
        request: Request<CommandOutputRequest>,
    ) -> Result<Response<Self::CommandOutputStream>, Status> {
        let req = request.into_inner();
        let domain_stream = self.orchestrator.register_admin(&req.admin_id).await?;

        let response_stream =
            domain_stream.map(|res| res.map(IntoProto::into_proto).map_err(Status::from));
        Ok(Response::new(Box::pin(response_stream)))
    }
}
