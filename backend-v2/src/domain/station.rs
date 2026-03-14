use async_trait::async_trait;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct Station {
    pub ip: String,
}

#[async_trait]
pub trait StationRepository: Send + Sync {
    async fn get(&self, ip: &str) -> Result<Station, AppError>;
    async fn get_all(&self) -> Result<Vec<Station>, AppError>;
    async fn upsert(&self, ip: &str) -> Result<(), AppError>;
    async fn delete(&self, ip: &str) -> Result<(), AppError>;
}
