mod convert;

use std::sync::Arc;

use async_trait::async_trait;
use loom_rpc::map::v1::{self as pb, map_service_server::MapService};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::domain::{MapElement, MapRepository};

pub struct MapHandler {
    map_repo: Arc<dyn MapRepository>,
}

impl MapHandler {
    pub fn new(map_repo: Arc<dyn MapRepository>) -> Self {
        Self { map_repo }
    }
}

#[async_trait]
impl MapService for MapHandler {
    async fn get_all_map_metadata(
        &self,
        _request: Request<()>,
    ) -> Result<Response<pb::GetAllMapMetadataResponse>, Status> {
        let maps = self.map_repo.get_all_metadata().await?;
        Ok(Response::new(pb::GetAllMapMetadataResponse {
            maps: maps.into_iter().map(Into::into).collect(),
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
                elements: map.elements.iter().map(Into::into).collect(),
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
                .map(MapElement::try_from)
                .collect::<Result<Vec<MapElement>, _>>()?;
            self.map_repo.upsert_elements(req.id, elements).await?;
        }

        Ok(Response::new(()))
    }
}
