use async_trait::async_trait;
use loom_core::map::{Map, MapElement, MapMetadata};
use uuid::Uuid;

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
        seat_id: Uuid,
        station_id: Option<String>,
    ) -> Result<Option<Uuid>, AppError>;
    async fn get_all_station_assignments(&self) -> Result<Vec<(Uuid, Option<String>)>, AppError>;
    async fn get_seat_id_by_ip(&self, ip: &str) -> Result<Option<Uuid>, AppError>;
}
