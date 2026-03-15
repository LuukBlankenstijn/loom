use async_trait::async_trait;

use crate::domain::types::Station;
use crate::error::AppError;

#[async_trait]
pub trait StationRepository: Send + Sync {
    async fn get(&self, ip: &str) -> Result<Station, AppError>;
    async fn get_all(&self) -> Result<Vec<Station>, AppError>;
    async fn upsert(&self, ip: &str) -> Result<(), AppError>;
    async fn delete(&self, ip: &str) -> Result<(), AppError>;
}
