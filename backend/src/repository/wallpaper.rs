use async_trait::async_trait;
use derive_more::derive::Constructor;
use futures::stream;
use sqlx::{FromRow, SqliteConnection, SqlitePool};

use crate::{
    domain::{WallpaperRepository, WallpaperStream},
    error::AppError,
};

#[derive(FromRow)]
struct WallpaperRow {
    pub mime_type: String,
    pub text_color: String,
    pub data: Vec<u8>,
}

impl From<WallpaperRow> for WallpaperStream {
    fn from(row: WallpaperRow) -> Self {
        let data = row.data;
        WallpaperStream {
            mime_type: row.mime_type,
            text_color: row.text_color,
            stream: Box::pin(stream::once(async move { Ok(data) })),
        }
    }
}

#[derive(Constructor)]
pub struct WallpaperRepo {
    pool: SqlitePool,
}

async fn insert_asset(
    conn: &mut SqliteConnection,
    data: &[u8],
    mime_type: &str,
    text_color: Option<String>,
) -> Result<i64, AppError> {
    let id = sqlx::query_scalar!(
        "INSERT INTO wallpapers (data, mime_type) VALUES ($1, $2) RETURNING id",
        data,
        mime_type
    )
    .fetch_one(&mut *conn)
    .await?;

    if let Some(text_color) = text_color {
        sqlx::query!(
            "UPDATE wallpapers SET text_color = $1 WHERE id = $2",
            text_color,
            id
        )
        .execute(conn)
        .await?;
    }

    Ok(id)
}

async fn delete_unreferenced_assets(conn: &mut SqliteConnection) -> Result<(), AppError> {
    sqlx::query!(
        "DELETE FROM wallpapers AS w
         WHERE NOT EXISTS (SELECT 1 FROM contest_wallpaper c WHERE c.wallpaper_id = w.id)
           AND NOT EXISTS (SELECT 1 FROM system_settings s WHERE s.default_wallpaper_id = w.id)"
    )
    .execute(conn)
    .await?;
    Ok(())
}

#[async_trait]
impl WallpaperRepository for WallpaperRepo {
    async fn get_for_contest(&self, contest_id: &str) -> Result<Option<WallpaperStream>, AppError> {
        let row = sqlx::query_as!(
            WallpaperRow,
            "SELECT w.mime_type, w.text_color, w.data
             FROM wallpapers w
             JOIN contest_wallpaper c ON c.wallpaper_id = w.id
             WHERE c.contest_id = $1",
            contest_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(WallpaperStream::from))
    }

    async fn set_for_contest(
        &self,
        contest_id: &str,
        data: &[u8],
        mime_type: &str,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let previous_color = sqlx::query_scalar!(
            "SELECT w.text_color
             FROM wallpapers w
             JOIN contest_wallpaper c ON c.wallpaper_id = w.id
             WHERE c.contest_id = $1",
            contest_id
        )
        .fetch_optional(&mut *tx)
        .await?;
        let wallpaper_id = insert_asset(&mut tx, data, mime_type, previous_color).await?;
        sqlx::query!(
            "INSERT INTO contest_wallpaper (contest_id, wallpaper_id) VALUES ($1, $2)
             ON CONFLICT (contest_id) DO UPDATE SET wallpaper_id = $2",
            contest_id,
            wallpaper_id
        )
        .execute(&mut *tx)
        .await?;
        delete_unreferenced_assets(&mut tx).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn clear_for_contest(&self, contest_id: &str) -> Result<(), AppError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        sqlx::query!(
            "DELETE FROM contest_wallpaper WHERE contest_id = $1",
            contest_id
        )
        .execute(&mut *tx)
        .await?;
        delete_unreferenced_assets(&mut tx).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn set_contest_text_color(&self, contest_id: &str, color: &str) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE wallpapers SET text_color = $1
             WHERE id = (SELECT wallpaper_id FROM contest_wallpaper WHERE contest_id = $2)",
            color,
            contest_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_default(&self) -> Result<Option<WallpaperStream>, AppError> {
        let row = sqlx::query_as!(
            WallpaperRow,
            "SELECT w.mime_type, w.text_color, w.data
             FROM wallpapers w
             JOIN system_settings s ON s.default_wallpaper_id = w.id"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(WallpaperStream::from))
    }

    async fn set_default(&self, data: &[u8], mime_type: &str) -> Result<(), AppError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let previous_color = sqlx::query_scalar!(
            "SELECT w.text_color
             FROM wallpapers w
             JOIN system_settings s ON s.default_wallpaper_id = w.id"
        )
        .fetch_optional(&mut *tx)
        .await?;
        let wallpaper_id = insert_asset(&mut tx, data, mime_type, previous_color).await?;
        sqlx::query!(
            "INSERT INTO system_settings (id, default_wallpaper_id) VALUES (1, $1)
             ON CONFLICT (id) DO UPDATE SET default_wallpaper_id = $1",
            wallpaper_id
        )
        .execute(&mut *tx)
        .await?;
        delete_unreferenced_assets(&mut tx).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn clear_default(&self) -> Result<(), AppError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        sqlx::query!("UPDATE system_settings SET default_wallpaper_id = NULL")
            .execute(&mut *tx)
            .await?;
        delete_unreferenced_assets(&mut tx).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn set_default_text_color(&self, color: &str) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE wallpapers SET text_color = $1
             WHERE id = (SELECT default_wallpaper_id FROM system_settings)",
            color
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
