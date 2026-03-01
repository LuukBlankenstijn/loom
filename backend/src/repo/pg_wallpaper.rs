use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::{Wallpaper, WallpaperRepository};
use crate::error::AppError;

pub struct PgWallpaperRepo(PgPool);

impl PgWallpaperRepo {
    pub fn new(pool: PgPool) -> Self {
        Self(pool)
    }
}

#[derive(sqlx::FromRow)]
struct WallpaperRow {
    contest_id: String,
    mime_type: String,
    updated_at: DateTime<Utc>,
    color: String,
    image_data: Vec<u8>,
}

impl From<WallpaperRow> for Wallpaper {
    fn from(r: WallpaperRow) -> Self {
        Self {
            contest_id: r.contest_id,
            mime_type: r.mime_type,
            updated_at: r.updated_at,
            text_color: r.color,
            data: r.image_data,
        }
    }
}

#[async_trait]
impl WallpaperRepository for PgWallpaperRepo {
    async fn set_wallpaper_data(
        &self,
        contest_id: &str,
        data: &[u8],
        mime_type: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO wallpapers (contest_id, image_data, mime_type, updated_at)
             VALUES ($1, $2, $3, NOW())
             ON CONFLICT (contest_id) DO UPDATE
             SET image_data = $2, mime_type = $3, updated_at = NOW()",
        )
        .bind(contest_id)
        .bind(data)
        .bind(mime_type)
        .execute(&self.0)
        .await?;
        Ok(())
    }

    async fn set_wallpaper_text_color(
        &self,
        contest_id: &str,
        color: &str,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE wallpapers SET color = $1 WHERE contest_id = $2")
            .bind(color)
            .bind(contest_id)
            .execute(&self.0)
            .await?;
        Ok(())
    }

    async fn delete_wallpaper(&self, contest_id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM wallpapers WHERE contest_id = $1")
            .bind(contest_id)
            .execute(&self.0)
            .await?;
        Ok(())
    }

    async fn get_wallpaper(&self, contest_id: &str) -> Result<Option<Wallpaper>, AppError> {
        let row = sqlx::query_as::<_, WallpaperRow>(
            "SELECT contest_id, mime_type, updated_at, color, image_data
             FROM wallpapers WHERE contest_id = $1",
        )
        .bind(contest_id)
        .fetch_optional(&self.0)
        .await?;

        Ok(row.map(Into::into))
    }

    async fn get_last_updated(&self, contest_id: &str) -> Result<Option<DateTime<Utc>>, AppError> {
        let row: Option<(DateTime<Utc>,)> =
            sqlx::query_as("SELECT updated_at FROM wallpapers WHERE contest_id = $1")
                .bind(contest_id)
                .fetch_optional(&self.0)
                .await?;
        Ok(row.map(|r| r.0))
    }
}
