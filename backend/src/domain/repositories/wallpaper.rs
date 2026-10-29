use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;

use crate::{domain::ContestRepository, error::AppError};

pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Vec<u8>, AppError>> + Send>>;

pub struct WallpaperStream {
    pub mime_type: String,
    pub stream: ByteStream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallpaperKind {
    Image,
    Youtube,
}

#[derive(Debug, Clone)]
pub enum WallpaperSource {
    Stored { mime_type: String },
    Url(String),
}

#[derive(Debug, Clone)]
pub struct Wallpaper {
    pub id: i32,
    pub name: String,
    pub kind: WallpaperKind,
    pub source: WallpaperSource,
    pub text_color: String,
}

pub struct WallpaperListing {
    pub wallpaper: Wallpaper,
    pub is_default: bool,
    pub contest_ids: Vec<String>,
}

#[async_trait]
pub trait WallpaperRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<WallpaperListing>, AppError>;
    async fn create_stored(
        &self,
        name: &str,
        data: &[u8],
        mime_type: &str,
    ) -> Result<Wallpaper, AppError>;
    async fn create_url(
        &self,
        name: &str,
        url: &str,
        kind: WallpaperKind,
    ) -> Result<Wallpaper, AppError>;
    async fn update(
        &self,
        id: i32,
        name: Option<&str>,
        text_color: Option<&str>,
    ) -> Result<(), AppError>;
    async fn delete(&self, id: i32) -> Result<(), AppError>;
    async fn get_content(&self, id: i32) -> Result<Option<WallpaperStream>, AppError>;

    async fn get_for_contest(&self, contest_id: &str) -> Result<Option<Wallpaper>, AppError>;
    async fn assign_contest(&self, contest_id: &str, id: Option<i32>) -> Result<(), AppError>;

    async fn get_default(&self) -> Result<Option<Wallpaper>, AppError>;
    async fn set_default(&self, id: Option<i32>) -> Result<(), AppError>;
}

pub async fn resolve_wallpaper(
    contest_repo: &dyn ContestRepository,
    wallpaper_repo: &dyn WallpaperRepository,
    contest_id: Option<String>,
) -> Result<Option<Wallpaper>, AppError> {
    let contest_id = match contest_id {
        Some(id) => Some(id),
        None => contest_repo.get_next_contest().await?.map(|c| c.id),
    };

    if let Some(contest_id) = contest_id
        && let Some(wallpaper) = wallpaper_repo.get_for_contest(&contest_id).await?
    {
        return Ok(Some(wallpaper));
    }

    wallpaper_repo.get_default().await
}
