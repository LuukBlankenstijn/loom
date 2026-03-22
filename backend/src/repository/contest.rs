use async_trait::async_trait;
use futures::stream;
use loom_core::contest::Contest;
use sqlx::{FromRow, PgPool};

use crate::{
    config::IcpcApiConfig,
    domain::{ContestRepository, WallpaperStream},
    error::AppError,
};

mod http;
mod pg;

#[async_trait]
trait InnerRepo: Send + Sync {
    async fn get_next_contest(&self) -> Result<Option<Contest>, AppError>;
}

#[derive(FromRow)]
struct WallpaperRow {
    pub mime_type: String,
    pub text_color: String,
    pub data: Vec<u8>,
}

pub struct ContestRepo {
    pool: PgPool,
    inner: Box<dyn InnerRepo>,
}

impl ContestRepo {
    pub fn new(pool: PgPool, client: reqwest::Client, config: Option<IcpcApiConfig>) -> Self {
        let inner: Box<dyn InnerRepo> = if let Some(config) = config {
            Box::new(http::HttpContestRepo::new(config, client))
        } else {
            Box::new(pg::PgContestRepo::new(pool.clone()))
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

    async fn set_wallpaper(
        &self,
        contest_id: &str,
        data: &[u8],
        mime_type: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO wallpapers (contest_id, data, mime_type)
             VALUES ($1, $2, $3)
             ON CONFLICT (contest_id) DO UPDATE
             SET data = $2, mime_type = $3",
            contest_id,
            data,
            mime_type
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn set_wallpaper_text_color(
        &self,
        contest_id: &str,
        color: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE wallpapers SET text_color = $1 WHERE contest_id = $2",
            color,
            contest_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete_wallpaper(&self, contest_id: &str) -> Result<(), AppError> {
        sqlx::query!("DELETE FROM wallpapers WHERE contest_id = $1", contest_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_wallpaper(&self, contest_id: &str) -> Result<Option<WallpaperStream>, AppError> {
        let row = sqlx::query_as!(
            WallpaperRow,
            "SELECT mime_type, text_color, data FROM wallpapers WHERE contest_id = $1",
            contest_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row.map(|r| {
            let data_payload = r.data;
            let stream = stream::once(async move { Ok(data_payload) });
            WallpaperStream {
                mime_type: r.mime_type,
                text_color: r.text_color,
                stream: Box::pin(stream),
            }
        }))
    }
}
