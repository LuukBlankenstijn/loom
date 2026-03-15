use std::pin::Pin;

use chrono::{DateTime, Utc};
use futures::Stream;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct Contest {
    pub id: String,
    pub name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Vec<u8>, AppError>> + Send>>;

pub struct WallpaperStream {
    pub mime_type: String,
    pub text_color: String,
    pub stream: ByteStream,
}
