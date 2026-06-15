use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::{Path, Query, State},
    http::Response,
    response::IntoResponse,
};
use derive_more::derive::Constructor;
use reqwest::header;
use serde::Deserialize;
use tokio_stream::StreamExt as _;

use crate::{
    domain::{ContestRepository, MapRepository, Orchestrator, TeamRepository},
    error::AppError,
    render,
};
use loom_core::map::MapElement;

#[derive(Clone, Constructor)]
pub struct HttpHandlerState {
    contest_repo: Arc<dyn ContestRepository>,
    team_repo: Arc<dyn TeamRepository>,
    map_repo: Arc<dyn MapRepository>,
    orchestrator: Arc<dyn Orchestrator>,
}

#[derive(Deserialize)]
pub struct WallpaperParams {
    pub ip: Option<String>,
    pub contest_id: Option<String>,
}

pub async fn wallpaper_handler(
    State(state): State<HttpHandlerState>,
    Query(query): Query<WallpaperParams>,
) -> Result<impl IntoResponse, AppError> {
    let contest_id = if let Some(id) = query.contest_id {
        id
    } else {
        let contest_option = state.contest_repo.get_next_contest().await?;
        contest_option
            .map(|c| c.id)
            .ok_or_else(|| AppError::NotFound("no upcoming contest found".to_string()))?
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
        state.team_repo.get_by_ip(&ip).await?.map(|t| t.name)
    } else {
        None
    };

    // create request builder
    let mut builder = Response::builder().header(header::CONTENT_TYPE, wallpaper.mime_type);

    // if teamname, set headers
    if let Some(team_name) = team_name {
        builder = builder.header("X-Wallpaper-Text", team_name);
    }
    builder = builder.header("X-Wallpaper-Text-Color", wallpaper.text_color);

    builder
        .body(Body::from_stream(body_stream))
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn next_contest(
    State(state): State<HttpHandlerState>,
) -> Result<impl IntoResponse, AppError> {
    let contest = state.contest_repo.get_next_contest().await?;
    let contest = contest.ok_or_else(|| AppError::NotFound("No next contest found".to_string()))?;

    Ok(Json(contest))
}

pub async fn team_info(
    State(state): State<HttpHandlerState>,
    Path(ip): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let team = state
        .team_repo
        .get_by_ip(&ip)
        .await?
        .ok_or(AppError::NotFound(format!("team with ip {} not found", ip)))?;

    Ok(Json(serde_json::json!({
        "name": team.name,
        "id": team.id,
        "ip": ip
    })))
}

pub async fn station_inventory(
    State(state): State<HttpHandlerState>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(
        state
            .orchestrator
            .get_state()
            .into_iter()
            .filter_map(|state_entry| {
                if state_entry.connected {
                    Some(state_entry.ip)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
    ))
}

#[derive(Deserialize)]
pub struct MapImageParams {
    pub ip: Option<String>,
    pub team_id: Option<String>,
    pub contest_id: Option<String>,
}

pub async fn map_image(
    State(state): State<HttpHandlerState>,
    Query(query): Query<MapImageParams>,
) -> Result<impl IntoResponse, AppError> {
    let contest_id = if let Some(id) = query.contest_id {
        id
    } else {
        state
            .contest_repo
            .get_next_contest()
            .await?
            .map(|c| c.id)
            .ok_or_else(|| AppError::NotFound("no upcoming contest found".to_string()))?
    };

    let map = state
        .map_repo
        .get_by_contest(&contest_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("no map for contest {contest_id}")))?;

    let target_ip: Option<String> = if let Some(ip) = query.ip {
        Some(ip)
    } else if let Some(team_id) = query.team_id {
        state.team_repo.get(&team_id).await?.and_then(|t| t.ip)
    } else {
        None
    };

    let highlight_seat_id = target_ip.as_deref().and_then(|ip| {
        map.elements.iter().find_map(|el| match el {
            MapElement::Seat(s) if s.ip.as_deref() == Some(ip) => Some(s.id),
            _ => None,
        })
    });

    let png = render::render_map_png(&map, highlight_seat_id)?;

    Response::builder()
        .header(header::CONTENT_TYPE, "image/png")
        .body(Body::from(png))
        .map_err(|e| AppError::Internal(e.to_string()))
}
