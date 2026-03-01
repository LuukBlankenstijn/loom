use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct Station {
    pub id: i32,
    pub ip: String,
    pub connected_at: DateTime<Utc>,
    pub disconnected_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait StationRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<Station>, AppError>;
    async fn upsert(&self, ip: &str) -> Result<(), AppError>;
    async fn update_disconnected_at(&self, ip: &str) -> Result<(), AppError>;
}
