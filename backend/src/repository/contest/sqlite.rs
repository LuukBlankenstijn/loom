use async_trait::async_trait;
use chrono::{DateTime, Utc};
use loom_core::contest::Contest;
use sqlx::SqlitePool;

use super::InnerRepo;
use crate::error::AppError;

#[derive(sqlx::FromRow)]
struct ContestRow {
    id: String,
    name: String,
    start_time: Option<DateTime<Utc>>,
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

pub struct SqliteContestRepo(SqlitePool);

impl SqliteContestRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self(pool)
    }
}

#[async_trait]
impl InnerRepo for SqliteContestRepo {
    async fn get_next_contest(&self) -> Result<Option<Contest>, AppError> {
        let row = sqlx::query_as!(
            ContestRow,
            r#"SELECT id, name,
                      start_time AS "start_time: DateTime<Utc>",
                      end_time AS "end_time: DateTime<Utc>"
               FROM contests
               WHERE unixepoch(end_time) > unixepoch()
               ORDER BY unixepoch(start_time) ASC NULLS LAST
               LIMIT 1"#
        )
        .fetch_optional(&self.0)
        .await?;
        Ok(row.map(Into::into))
    }
}
