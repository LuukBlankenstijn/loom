use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::{Station, StationRepository};
use crate::error::AppError;

#[derive(sqlx::FromRow)]
struct StationRow {
    id: i64,
    ip: String,
}

impl From<StationRow> for Station {
    fn from(r: StationRow) -> Self {
        Self {
            id: r.id as i32,
            ip: r.ip,
        }
    }
}

pub struct PgStationRepo(PgPool);

impl PgStationRepo {
    pub fn new(pool: PgPool) -> Self {
        Self(pool)
    }
}

#[async_trait]
impl StationRepository for PgStationRepo {
    async fn get_all(&self) -> Result<Vec<Station>, AppError> {
        let rows: Vec<StationRow> = sqlx::query_as("SELECT id, ip FROM stations")
            .fetch_all(&self.0)
            .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn upsert(&self, ip: &str) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO stations (ip)
         VALUES ($1)
         ON CONFLICT (ip) DO NOTHING",
        )
        .bind(ip)
        .execute(&self.0)
        .await?;

        Ok(())
    }

    async fn delete(&self, ids: &[i32]) -> Result<(), AppError> {
        sqlx::query("DELETE FROM stations WHERE id = ANY($1)")
            .bind(ids)
            .execute(&self.0)
            .await?;

        Ok(())
    }
}
