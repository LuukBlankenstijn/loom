use async_trait::async_trait;
use chrono::{DateTime, Utc};
use loom_core::contest::Contest;
use sqlx::PgPool;

use super::InnerRepo;
use crate::error::AppError;

#[derive(sqlx::FromRow)]
struct ContestRow {
    id: String,
    name: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
}

impl From<ContestRow> for Contest {
    fn from(r: ContestRow) -> Self {
        Self {
            id: r.id,
            name: r.name,
            start_time: r.start_time,
            end_time: r.end_time,
        }
    }
}

pub struct PgContestRepo(PgPool);

impl PgContestRepo {
    pub fn new(pool: PgPool) -> Self {
        Self(pool)
    }
}

#[async_trait]
impl InnerRepo for PgContestRepo {
    async fn get_next_contest(&self) -> Result<Option<Contest>, AppError> {
        let row = sqlx::query_as!(
            ContestRow,
            "SELECT id, name, start_time, end_time FROM contests
             WHERE end_time > NOW()
             ORDER BY start_time ASC
             LIMIT 1"
        )
        .fetch_optional(&self.0)
        .await?;
        Ok(row.map(Into::into))
    }
}
