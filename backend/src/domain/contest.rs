use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct Contest {
    pub id: String,
    pub name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[async_trait]
pub trait ContestRepository: Send + Sync {
    async fn get_next_contest(&self) -> Result<Option<Contest>, AppError>;
}
