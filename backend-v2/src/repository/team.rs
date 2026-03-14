mod http;
mod pg;

use crate::{
    config::IcpcApiConfig,
    domain::{Team, TeamRepository},
    error::AppError,
};
use async_trait::async_trait;
use sqlx::PgPool;

pub struct TeamRepo {
    inner: Box<dyn TeamRepository>,
}

impl TeamRepo {
    pub fn new(pool: PgPool, client: reqwest::Client, config: Option<IcpcApiConfig>) -> Self {
        let inner: Box<dyn TeamRepository> = if let Some(config) = config {
            Box::new(http::HttpTeamRepo::new(config, client))
        } else {
            Box::new(pg::PgTeamRepo::new(pool))
        };
        Self { inner }
    }
}

#[async_trait]
impl TeamRepository for TeamRepo {
    async fn set_ip(&self, team_id: &str, ip: Option<&str>) -> Result<Option<String>, AppError> {
        self.inner.set_ip(team_id, ip).await
    }

    async fn get_all(&self, contest_id: &str) -> Result<Vec<Team>, AppError> {
        self.inner.get_all(contest_id).await
    }

    async fn get_by_ip(&self, ip: &str) -> Result<Team, AppError> {
        self.inner.get_by_ip(ip).await
    }
}
