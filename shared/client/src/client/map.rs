use crate::{
    client::Client,
    convert::map::prelude::{ToProto, TryFromProto},
};

use loom_map_types::MapElement;
use loom_rpc::map::v1::{AssignStationRequest, GetMapRequest, UpdateMapRequest};
use uuid::Uuid;

pub trait MapClient {
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
    fn assign_station_to_seat(
        &self,
        station_ip: String,
        seat_id: Option<Uuid>,
    ) -> impl Future<Output = Result<(), String>>;
}

impl MapClient for Client {
    async fn get_map_elements(&self, map_id: i32) -> Result<Vec<MapElement>, String> {
        Ok(self
            .map_client
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
        self.map_client
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

    async fn assign_station_to_seat(
        &self,
        station_ip: String,
        seat_id: Option<Uuid>,
    ) -> Result<(), String> {
        self.map_client
            .clone()
            .assign_station_to_seat(AssignStationRequest {
                station_ip,
                seat_id: seat_id.map(|id| id.to_string()),
            })
            .await
            .map_err(|e| e.to_string())
            .map(|_| ())
    }
}
