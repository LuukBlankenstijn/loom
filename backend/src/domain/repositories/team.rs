use async_trait::async_trait;
use loom_core::team::Team;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct IpChange {
    pub old: Option<String>,
    pub new: Option<String>,
}

#[async_trait]
pub trait TeamRepository: Send + Sync {
    /// Sets or removes the ip for some team. Returns the team's previous and new ip
    /// so callers can react to both sides of the change.
    async fn set_ip(&self, team_id: &str, ip: Option<&str>) -> Result<IpChange, AppError>;
    async fn get_all(&self, contest_id: &str) -> Result<Vec<Team>, AppError>;
    async fn get_by_ip(&self, ip: &str) -> Result<Option<Team>, AppError>;
}
