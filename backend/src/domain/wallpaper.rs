use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct Wallpaper {
    pub contest_id: String,
    pub mime_type: String,
    pub updated_at: DateTime<Utc>,
    pub text_color: String,
    pub data: Vec<u8>,
}

#[async_trait]
pub trait WallpaperRepository: Send + Sync {
    async fn set_wallpaper_data(
        &self,
        contest_id: &str,
        data: &[u8],
        mime_type: &str,
    ) -> Result<(), AppError>;
    async fn set_wallpaper_text_color(&self, contest_id: &str, color: &str)
    -> Result<(), AppError>;
    async fn delete_wallpaper(&self, contest_id: &str) -> Result<(), AppError>;
    async fn get_wallpaper(&self, contest_id: &str) -> Result<Option<Wallpaper>, AppError>;
    async fn get_last_updated(&self, contest_id: &str) -> Result<Option<DateTime<Utc>>, AppError>;
}
