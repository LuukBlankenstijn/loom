use async_trait::async_trait;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub ip: Option<String>,
}

#[async_trait]
pub trait TeamRepository: Send + Sync {
    async fn set_ip(&self, team_id: &str, ip: Option<&str>) -> Result<(), AppError>;
    async fn get_all(&self, contest_id: &str) -> Result<Vec<Team>, AppError>;
    async fn get_by_ip(&self, ip: &str) -> Result<Option<Team>, AppError>;
}
