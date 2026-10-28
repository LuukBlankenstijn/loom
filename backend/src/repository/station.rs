use async_trait::async_trait;
use loom_core::station::Station;
use sqlx::PgPool;

use crate::domain::StationRepository;
use crate::error::AppError;

#[derive(sqlx::FromRow)]
struct StationRow {
    ip: String,
    reported_label: Option<String>,
    label_override: Option<String>,
}

impl From<StationRow> for Station {
    fn from(r: StationRow) -> Self {
        Self {
            ip: r.ip,
            reported_label: r.reported_label,
            label_override: r.label_override,
        }
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
        let row = sqlx::query_as!(
            StationRow,
            "SELECT ip, reported_label, label_override FROM stations WHERE ip = $1",
            ip
        )
        .fetch_one(&self.0)
        .await?;

        Ok(row.into())
    }

    async fn get_all(&self) -> Result<Vec<Station>, AppError> {
        let rows = sqlx::query_as!(
            StationRow,
            "SELECT ip, reported_label, label_override FROM stations"
        )
        .fetch_all(&self.0)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn upsert(&self, ip: &str, reported_label: Option<&str>) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO stations (ip, reported_label) VALUES ($1, $2)
             ON CONFLICT (ip) DO UPDATE SET reported_label = EXCLUDED.reported_label",
            ip,
            reported_label
        )
        .execute(&self.0)
        .await?;

        Ok(())
    }

    async fn set_label_override(&self, ip: &str, label: Option<&str>) -> Result<(), AppError> {
        let result = sqlx::query!(
            "UPDATE stations SET label_override = $2 WHERE ip = $1",
            ip,
            label
        )
        .execute(&self.0)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("station {ip} not found")));
        }

        Ok(())
    }

    async fn delete(&self, ip: &str) -> Result<(), AppError> {
        sqlx::query!("DELETE FROM stations WHERE ip = $1", ip)
            .execute(&self.0)
            .await?;

        Ok(())
    }
}
