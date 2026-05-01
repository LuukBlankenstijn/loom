use std::sync::Arc;

use async_trait::async_trait;
use derive_more::derive::Constructor;
use loom_core::{event::broadcast::StationAssignment, map::MapElement};
use loom_proto_bridge::{IntoProto, TryIntoCore};
use loom_rpc::map::v1::{self as pb, map_service_server::MapService};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{
    domain::{MapRepository, Orchestrator, TeamRepository},
    error::AppError,
};

#[derive(Constructor)]
pub struct MapHandler {
    orchestrator: Arc<dyn Orchestrator>,
    map_repo: Arc<dyn MapRepository>,
    team_repo: Arc<dyn TeamRepository>,
}

#[async_trait]
impl MapService for MapHandler {
    async fn get_all_map_metadata(
        &self,
        _request: Request<()>,
    ) -> Result<Response<pb::GetAllMapMetadataResponse>, Status> {
        let maps = self.map_repo.get_all_metadata().await?;
        Ok(Response::new(pb::GetAllMapMetadataResponse {
            maps: maps.into_iter().map(IntoProto::into_proto).collect(),
        }))
    }

    async fn create_map(
        &self,
        request: Request<pb::CreateMapRequest>,
    ) -> Result<Response<pb::MapResponse>, Status> {
        let name = request.into_inner().name;
        let map = self.map_repo.create_map(&name).await?;
        Ok(Response::new(pb::MapResponse {
            map: Some(pb::MapMetadata { id: map.id, name }),
            elements: vec![],
        }))
    }

    async fn get_map(
        &self,
        request: Request<pb::GetMapRequest>,
    ) -> Result<Response<pb::MapResponse>, Status> {
        let req = request.into_inner();
        let map = self.map_repo.get(req.id).await?;
        match map {
            Some(map) => Ok(Response::new(pb::MapResponse {
                map: Some(pb::MapMetadata {
                    id: map.id,
                    name: map.name,
                }),
                elements: map.elements.iter().map(IntoProto::into_proto).collect(),
            })),
            None => todo!(),
        }
    }

    async fn update_map(
        &self,
        request: Request<pb::UpdateMapRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();

        let deleted_ids: Vec<Uuid> = req
            .deleted
            .iter()
            .filter_map(|s| Uuid::parse_str(s).ok())
            .collect();

        if !deleted_ids.is_empty() {
            self.map_repo.delete_elements(&deleted_ids).await?;
        }

        if !req.updated.is_empty() {
            let elements = req
                .updated
                .into_iter()
                .map(TryIntoCore::try_into_core)
                .collect::<Result<Vec<MapElement>, _>>()?;
            self.map_repo.upsert_elements(req.id, elements).await?;
        }

        Ok(Response::new(()))
    }

    async fn assign_station_to_seat(
        &self,
        request: Request<pb::AssignStationRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let seat_id = Uuid::try_parse(&req.seat_id)
            .map_err(|_| AppError::InvalidArgument("invalid seat id".to_string()))?;
        let old = self
            .map_repo
            .assign_station_to_seat(seat_id, req.station_ip.clone())
            .await?;

        let team_name = match req.station_ip.as_deref() {
            Some(ip) => self.team_repo.get_by_ip(ip).await?.map(|t| t.name),
            None => None,
        };

        let mut assignments = vec![StationAssignment {
            seat_id,
            station_ip: req.station_ip,
            team_name,
        }];

        if let Some(old) = old {
            assignments.push(StationAssignment {
                seat_id: old,
                station_ip: None,
                team_name: None,
            });
        }

        self.orchestrator.broadcast(assignments.into());

        Ok(().into())
    }
}
