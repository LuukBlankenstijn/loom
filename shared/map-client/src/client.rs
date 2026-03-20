use crate::convert::prelude::ToProto;

use super::convert::prelude::TryFromProto;
use loom_map_types::MapElement;
use loom_rpc::map::v1::{GetMapRequest, UpdateMapRequest, map_service_client::MapServiceClient};
#[cfg(not(target_arch = "wasm32"))]
use tonic::transport::Channel;
#[cfg(target_arch = "wasm32")]
use tonic_web_wasm_client::Client as Channel;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct MapClient {
    inner: MapServiceClient<Channel>,
}

impl MapClient {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: MapServiceClient::new(channel),
        }
    }
}

pub trait MapClientExt {
    fn get_map_elements(
        &self,
        map_id: i32,
    ) -> impl Future<Output = Result<Vec<MapElement>, String>>;
    fn update_map(
        &self,
        map_id: i32,
        deleted: Vec<Uuid>,
        updated: Vec<MapElement>,
    ) -> impl Future<Output = Result<(), String>>;
}

impl MapClientExt for MapClient {
    async fn get_map_elements(&self, map_id: i32) -> Result<Vec<MapElement>, String> {
        Ok(self
            .inner
            .clone()
            .get_map(GetMapRequest { id: map_id })
            .await
            .map_err(|e| e.to_string())?
            .into_inner()
            .elements
            .into_iter()
            .filter_map(|e| MapElement::try_from_proto(e.element?).ok())
            .collect())
    }

    async fn update_map(
        &self,
        map_id: i32,
        deleted: Vec<Uuid>,
        updated: Vec<MapElement>,
    ) -> Result<(), String> {
        self.inner
            .clone()
            .update_map(UpdateMapRequest {
                id: map_id,
                deleted: deleted.iter().map(|uuid| uuid.to_string()).collect(),
                updated: updated.into_iter().map(|u| u.to_proto()).collect(),
            })
            .await
            .map_err(|e| e.to_string())
            .map(|_| ())
    }
}
