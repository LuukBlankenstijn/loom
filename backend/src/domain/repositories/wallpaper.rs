use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;

use crate::error::AppError;

pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Vec<u8>, AppError>> + Send>>;

pub struct WallpaperStream {
    pub mime_type: String,
    pub stream: ByteStream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallpaperKind {
    Image,
    Video,
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
