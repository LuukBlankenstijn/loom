use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::StationAssignment;
use crate::domain::types::{Map, MapElement, MapMetadata};
use crate::error::AppError;

#[async_trait]
pub trait MapRepository: Send + Sync {
    async fn get(&self, map_id: i32) -> Result<Option<Map>, AppError>;
    async fn get_by_contest(&self, contest_id: &str) -> Result<Option<Map>, AppError>;

    async fn create_map(&self, name: &str) -> Result<Map, AppError>;

    async fn get_all_metadata(&self) -> Result<Vec<MapMetadata>, AppError>;

    async fn delete_elements(&self, element_ids: &[Uuid]) -> Result<(), AppError>;
    async fn upsert_elements(&self, map_id: i32, elements: Vec<MapElement>)
    -> Result<(), AppError>;

    async fn assign_station_to_seat(
        &self,
        station_ip: String,
        seat_id: Option<Uuid>,
    ) -> Result<(), AppError>;
    async fn get_all_station_assignments(
        &self,
        map_id: Option<i32>,
    ) -> Result<Vec<StationAssignment>, AppError>;
}
