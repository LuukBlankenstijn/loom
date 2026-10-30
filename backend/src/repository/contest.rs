use async_trait::async_trait;
use loom_core::contest::Contest;
use sqlx::SqlitePool;

use crate::{config::IcpcApiConfig, domain::ContestRepository, error::AppError};

mod http;
mod sqlite;

#[async_trait]
trait InnerRepo: Send + Sync {
    async fn get_next_contest(&self) -> Result<Option<Contest>, AppError>;
}

pub struct ContestRepo {
    pool: SqlitePool,
    inner: Box<dyn InnerRepo>,
}

impl ContestRepo {
    pub fn new(pool: SqlitePool, client: reqwest::Client, config: Option<IcpcApiConfig>) -> Self {
        let inner: Box<dyn InnerRepo> = if let Some(config) = config {
            Box::new(http::HttpContestRepo::new(config, client))
        } else {
            Box::new(sqlite::SqliteContestRepo::new(pool.clone()))
        };
        Self { pool, inner }
    }
}

#[async_trait]
impl ContestRepository for ContestRepo {
    async fn get_next_contest(&self) -> Result<Option<Contest>, AppError> {
        self.inner.get_next_contest().await
    }

    async fn set_map(&self, contest_id: &str, map_id: i32) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO contest_map_contest (contest_id, map_id) VALUES ($1, $2)
             ON CONFLICT (contest_id) DO UPDATE SET map_id = $2",
            contest_id,
            map_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
