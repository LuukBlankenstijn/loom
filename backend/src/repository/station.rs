use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::{Station, StationRepository};
use crate::error::AppError;

#[derive(sqlx::FromRow)]
struct StationRow {
    ip: String,
}

impl From<StationRow> for Station {
    fn from(r: StationRow) -> Self {
        Self { ip: r.ip }
    }
}

pub struct StationRepo(PgPool);

impl StationRepo {
    pub fn new(pool: PgPool) -> Self {
        Self(pool)
    }
}

#[async_trait]
impl StationRepository for StationRepo {
    async fn get(&self, ip: &str) -> Result<Station, AppError> {
        let row = sqlx::query_as!(StationRow, "SELECT ip FROM stations WHERE ip = $1", ip)
            .fetch_one(&self.0)
            .await?;

        Ok(row.into())
    }

    async fn get_all(&self) -> Result<Vec<Station>, AppError> {
        let rows = sqlx::query_as!(StationRow, "SELECT ip FROM stations")
            .fetch_all(&self.0)
            .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn upsert(&self, ip: &str) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO stations (ip) VALUES ($1) ON CONFLICT (ip) DO NOTHING",
            ip
        )
        .execute(&self.0)
        .await?;

        Ok(())
    }

    async fn delete(&self, ip: &str) -> Result<(), AppError> {
        sqlx::query!("DELETE FROM stations WHERE ip = $1", ip)
            .execute(&self.0)
            .await?;

        Ok(())
    }
}
