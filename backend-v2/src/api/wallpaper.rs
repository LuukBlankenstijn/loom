use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Query, State},
    http::Response,
    response::IntoResponse,
};
use derive_more::derive::Constructor;
use reqwest::header;
use serde::Deserialize;
use tokio_stream::StreamExt as _;

use crate::{
    domain::{ContestRepository, TeamRepository},
    error::AppError,
};

#[derive(Clone, Constructor)]
pub struct WallpaperHandler {
    contest_repo: Arc<dyn ContestRepository>,
    team_repo: Arc<dyn TeamRepository>,
}

#[derive(Deserialize)]
pub struct WallpaperParams {
    pub ip: Option<String>,
    pub contest_id: Option<String>,
}

pub async fn wallpaper_handler(
    State(state): State<WallpaperHandler>,
    Query(query): Query<WallpaperParams>,
) -> Result<impl IntoResponse, AppError> {
    let contest_id = if let Some(id) = query.contest_id {
        id
    } else {
        let contest_option = state.contest_repo.get_next_contest().await?;
        contest_option
            .map(|c| c.id)
            .ok_or_else(|| AppError::NotFound("no upcomming contest found".to_string()))?
    };
    // get the wallpaper
    let wallpaper = state
        .contest_repo
        .get_wallpaper(&contest_id)
        .await?
        .ok_or_else(|| AppError::NotFound("no wallpaper found".to_string()))?;
    // create stream from database stream
    let body_stream = wallpaper
        .stream
        .map(|res| res.map(bytes::Bytes::from).map_err(std::io::Error::other));

    // try to get team name
    let team_name = if let Some(ip) = query.ip {
        let team = state.team_repo.get_by_ip(&ip).await?;
        Some(team.name)
    } else {
        None
    };

    // create request builder
    let mut builder = Response::builder().header(header::CONTENT_TYPE, wallpaper.mime_type);

    // if teamname, set headers
    if let Some(team_name) = team_name {
        builder = builder.header("X-Wallpaper-Text", team_name);
        builder = builder.header("X-Wallpaper-Text-Color", wallpaper.text_color);
    }

    builder
        .body(Body::from_stream(body_stream))
        .map_err(|e| AppError::Internal(e.to_string()))
}
