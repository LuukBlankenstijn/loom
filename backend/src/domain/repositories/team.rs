use async_trait::async_trait;
use loom_core::team::Team;

use crate::error::AppError;

#[async_trait]
pub trait TeamRepository: Send + Sync {
    /// Sets or removes the ip for some team. Returns the old ip or the new ip, but is None if there
    /// is no old ip
    async fn set_ip(&self, team_id: &str, ip: Option<&str>) -> Result<Option<String>, AppError>;
    async fn get_all(&self, contest_id: &str) -> Result<Vec<Team>, AppError>;
    async fn get_by_ip(&self, ip: &str) -> Result<Option<Team>, AppError>;
}
