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
    async fn get_by_id(&self, id: i32) -> Result<Station, AppError> {
        let row: StationRow = sqlx::query_as("SELECT * FROM STATIONS WHERE id = $1")
            .bind(id)
            .fetch_one(&self.0)
            .await?;

        Ok(row.into())
    }

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

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        sqlx::query("DELETE FROM stations WHERE id = $1")
            .bind(id)
            .execute(&self.0)
            .await?;

        Ok(())
    }
}
