use async_trait::async_trait;

use crate::domain::types::{Contest, WallpaperStream};
use crate::error::AppError;

#[async_trait]
pub trait ContestRepository: Send + Sync {
    async fn get_next_contest(&self) -> Result<Option<Contest>, AppError>;
    async fn set_map(&self, contest_id: &str, map_id: i32) -> Result<(), AppError>;

    async fn set_wallpaper(
        &self,
        contest_id: &str,
        data: &[u8],
        mime_type: &str,
    ) -> Result<(), AppError>;
    async fn set_wallpaper_text_color(&self, contest_id: &str, color: &str)
    -> Result<(), AppError>;
    async fn delete_wallpaper(&self, contest_id: &str) -> Result<(), AppError>;
    async fn get_wallpaper(&self, contest_id: &str) -> Result<Option<WallpaperStream>, AppError>;
}
