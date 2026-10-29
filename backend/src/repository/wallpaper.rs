use std::collections::HashMap;

use async_trait::async_trait;
use derive_more::derive::Constructor;
use futures::stream;
use sqlx::{FromRow, SqlitePool, sqlite::SqliteQueryResult};

use crate::{
    domain::{
        Wallpaper, WallpaperKind, WallpaperListing, WallpaperRepository, WallpaperSource,
        WallpaperStream,
    },
    error::AppError,
};

#[derive(FromRow)]
struct WallpaperRow {
    id: i32,
    name: String,
    kind: String,
    mime_type: Option<String>,
    url: Option<String>,
    text_color: String,
}

impl TryFrom<WallpaperRow> for Wallpaper {
    type Error = AppError;

    fn try_from(row: WallpaperRow) -> Result<Self, AppError> {
        let kind = match row.kind.as_str() {
            "image" => WallpaperKind::Image,
            "youtube" => WallpaperKind::Youtube,
            other => {
                return Err(AppError::Internal(format!(
                    "unknown wallpaper kind {other}"
                )));
            }
        };
        let source = match (row.mime_type, row.url) {
            (Some(mime_type), None) => WallpaperSource::Stored { mime_type },
            (None, Some(url)) => WallpaperSource::Url(url),
            _ => {
                return Err(AppError::Internal(format!(
                    "wallpaper {} has no single source",
                    row.id
                )));
            }
        };
        Ok(Wallpaper {
            id: row.id,
            name: row.name,
            kind,
            source,
            text_color: row.text_color,
        })
    }
}

fn kind_column(kind: WallpaperKind) -> &'static str {
    match kind {
        WallpaperKind::Image => "image",
        WallpaperKind::Youtube => "youtube",
    }
}

fn require_row(result: SqliteQueryResult, id: i32) -> Result<(), AppError> {
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("wallpaper {id} not found")));
    }
    Ok(())
}

#[derive(Constructor)]
pub struct WallpaperRepo {
    pool: SqlitePool,
}

#[async_trait]
impl WallpaperRepository for WallpaperRepo {
    async fn list(&self) -> Result<Vec<WallpaperListing>, AppError> {
        let rows = sqlx::query!(
            r#"SELECT w.id AS "id: i32", w.name, w.kind, w.mime_type, w.url, w.text_color,
                      EXISTS (SELECT 1 FROM system_settings s WHERE s.default_wallpaper_id = w.id)
                          AS "is_default: bool"
               FROM wallpapers w
               ORDER BY w.id DESC"#
        )
        .fetch_all(&self.pool)
        .await?;

        let assignments = sqlx::query!(
            r#"SELECT contest_id, wallpaper_id AS "wallpaper_id: i32"
               FROM contest_wallpaper
               ORDER BY contest_id"#
        )
        .fetch_all(&self.pool)
        .await?;
        let mut contest_ids: HashMap<i32, Vec<String>> = HashMap::new();
        for assignment in assignments {
            contest_ids
                .entry(assignment.wallpaper_id)
                .or_default()
                .push(assignment.contest_id);
        }

        rows.into_iter()
            .map(|row| {
                let wallpaper = Wallpaper::try_from(WallpaperRow {
                    id: row.id,
                    name: row.name,
                    kind: row.kind,
                    mime_type: row.mime_type,
                    url: row.url,
                    text_color: row.text_color,
                })?;
                Ok(WallpaperListing {
                    contest_ids: contest_ids.remove(&wallpaper.id).unwrap_or_default(),
                    is_default: row.is_default,
                    wallpaper,
                })
            })
            .collect()
    }

    async fn create_stored(
        &self,
        name: &str,
        data: &[u8],
        mime_type: &str,
    ) -> Result<Wallpaper, AppError> {
        sqlx::query_as!(
            WallpaperRow,
            r#"INSERT INTO wallpapers (name, kind, data, mime_type) VALUES ($1, 'image', $2, $3)
               RETURNING id AS "id: i32", name, kind, mime_type, url, text_color"#,
            name,
            data,
            mime_type
        )
        .fetch_one(&self.pool)
        .await?
        .try_into()
    }

    async fn create_url(
        &self,
        name: &str,
        url: &str,
        kind: WallpaperKind,
    ) -> Result<Wallpaper, AppError> {
        let kind = kind_column(kind);
        sqlx::query_as!(
            WallpaperRow,
            r#"INSERT INTO wallpapers (name, kind, url) VALUES ($1, $2, $3)
               RETURNING id AS "id: i32", name, kind, mime_type, url, text_color"#,
            name,
            kind,
            url
        )
        .fetch_one(&self.pool)
        .await?
        .try_into()
    }

    async fn update(
        &self,
        id: i32,
        name: Option<&str>,
        text_color: Option<&str>,
    ) -> Result<(), AppError> {
        let result = sqlx::query!(
            "UPDATE wallpapers
             SET name = COALESCE($2, name), text_color = COALESCE($3, text_color)
             WHERE id = $1",
            id,
            name,
            text_color
        )
        .execute(&self.pool)
        .await?;
        require_row(result, id)
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        let result = sqlx::query!("DELETE FROM wallpapers WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        require_row(result, id)
    }

    async fn get_content(&self, id: i32) -> Result<Option<WallpaperStream>, AppError> {
        let row = sqlx::query!(
            r#"SELECT mime_type AS "mime_type!", data AS "data!"
               FROM wallpapers
               WHERE id = $1 AND data IS NOT NULL"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| {
            let data = row.data;
            WallpaperStream {
                mime_type: row.mime_type,
                stream: Box::pin(stream::once(async move { Ok(data) })),
            }
        }))
    }

    async fn get_for_contest(&self, contest_id: &str) -> Result<Option<Wallpaper>, AppError> {
        sqlx::query_as!(
            WallpaperRow,
            r#"SELECT w.id AS "id: i32", w.name, w.kind, w.mime_type, w.url, w.text_color
               FROM wallpapers w
               JOIN contest_wallpaper c ON c.wallpaper_id = w.id
               WHERE c.contest_id = $1"#,
            contest_id
        )
        .fetch_optional(&self.pool)
        .await?
        .map(Wallpaper::try_from)
        .transpose()
    }

    async fn assign_contest(&self, contest_id: &str, id: Option<i32>) -> Result<(), AppError> {
        let Some(id) = id else {
            sqlx::query!(
                "DELETE FROM contest_wallpaper WHERE contest_id = $1",
                contest_id
            )
            .execute(&self.pool)
            .await?;
            return Ok(());
        };

        let result = sqlx::query!(
            "INSERT INTO contest_wallpaper (contest_id, wallpaper_id)
             SELECT $1, id FROM wallpapers WHERE id = $2
             ON CONFLICT (contest_id) DO UPDATE SET wallpaper_id = EXCLUDED.wallpaper_id",
            contest_id,
            id
        )
        .execute(&self.pool)
        .await?;
        require_row(result, id)
    }

    async fn get_default(&self) -> Result<Option<Wallpaper>, AppError> {
        sqlx::query_as!(
            WallpaperRow,
            r#"SELECT w.id AS "id: i32", w.name, w.kind, w.mime_type, w.url, w.text_color
               FROM wallpapers w
               JOIN system_settings s ON s.default_wallpaper_id = w.id"#
        )
        .fetch_optional(&self.pool)
        .await?
        .map(Wallpaper::try_from)
        .transpose()
    }

    async fn set_default(&self, id: Option<i32>) -> Result<(), AppError> {
        let Some(id) = id else {
            sqlx::query!("UPDATE system_settings SET default_wallpaper_id = NULL")
                .execute(&self.pool)
                .await?;
            return Ok(());
        };

        let result = sqlx::query!(
            "UPDATE system_settings SET default_wallpaper_id = $1
             WHERE EXISTS (SELECT 1 FROM wallpapers WHERE id = $1)",
            id
        )
        .execute(&self.pool)
        .await?;
        require_row(result, id)
    }
}
