use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::{Station, StationRepository};
use crate::error::AppError;

#[derive(sqlx::FromRow)]
struct StationRow {
    id: i64,
    ip: String,
    connected_at: DateTime<Utc>,
    disconnected_at: Option<DateTime<Utc>>,
}

impl From<StationRow> for Station {
    fn from(r: StationRow) -> Self {
        Self {
            id: r.id as i32,
            ip: r.ip,
            connected_at: r.connected_at,
            disconnected_at: r.disconnected_at,
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
        let rows: Vec<StationRow> =
            sqlx::query_as("SELECT id, ip, connected_at, disconnected_at FROM stations")
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
}
