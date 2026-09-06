use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::{Path, Query, State},
    http::HeaderName,
    response::{AppendHeaders, IntoResponse},
};
use chrono::{DateTime, Utc};
use derive_more::derive::Constructor;
use reqwest::header;
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt as _;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    api::ErrorResponse,
    domain::{ContestRepository, MapRepository, Orchestrator, TeamRepository},
    error::AppError,
    render,
};
use loom_core::contest::Contest;
use loom_core::map::MapElement;

#[derive(Clone, Constructor)]
pub struct HttpHandlerState {
    contest_repo: Arc<dyn ContestRepository>,
    team_repo: Arc<dyn TeamRepository>,
    map_repo: Arc<dyn MapRepository>,
    orchestrator: Arc<dyn Orchestrator>,
}

impl HttpHandlerState {
    async fn resolve_contest_id(&self, requested: Option<String>) -> Result<String, AppError> {
        if let Some(id) = requested {
            return Ok(id);
        }
        self.contest_repo
            .get_next_contest()
            .await?
            .map(|contest| contest.id)
            .ok_or_else(|| AppError::NotFound("no upcoming contest found".to_string()))
    }

    async fn resolve_ip(
        &self,
        ip: Option<String>,
        team_id: Option<String>,
    ) -> Result<Option<String>, AppError> {
        if ip.is_some() {
            return Ok(ip);
        }
        match team_id {
            Some(team_id) => Ok(self.team_repo.get(&team_id).await?.and_then(|team| team.ip)),
            None => Ok(None),
        }
    }
}

pub fn router(state: HttpHandlerState) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(wallpaper))
        .routes(routes!(next_contest))
        .routes(routes!(team_info))
        .routes(routes!(map_image))
        .routes(routes!(station_inventory))
        .with_state(state)
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct WallpaperParams {
    /// Station address used to look up the team name rendered on the wallpaper.
    pub ip: Option<String>,
    /// Defaults to the next upcoming contest.
    pub contest_id: Option<String>,
}

#[utoipa::path(
    get,
    path = "/wallpaper",
    tag = "station",
    params(WallpaperParams),
    responses(
        (status = 200, description = "Wallpaper image bytes", content_type = "image/*",
         headers(
            ("x-wallpaper-text-color" = String, description = "Colour for text drawn over the wallpaper"),
            ("x-wallpaper-text" = String, description = "Team name; absent when no team matches ip")
         )),
        (status = 404, description = "No upcoming contest, or no wallpaper for the contest", body = ErrorResponse),
    )
)]
pub async fn wallpaper(
    State(state): State<HttpHandlerState>,
    Query(query): Query<WallpaperParams>,
) -> Result<impl IntoResponse, AppError> {
    let contest_id = state.resolve_contest_id(query.contest_id).await?;

    let wallpaper = state
        .contest_repo
        .get_wallpaper(&contest_id)
        .await?
        .ok_or_else(|| AppError::NotFound("no wallpaper found".to_string()))?;

    let body_stream = wallpaper
        .stream
        .map(|res| res.map(bytes::Bytes::from).map_err(std::io::Error::other));

    let mut headers = vec![
        (header::CONTENT_TYPE, wallpaper.mime_type),
        (
            HeaderName::from_static("x-wallpaper-text-color"),
            wallpaper.text_color,
        ),
    ];

    if let Some(ip) = query.ip
        && let Some(team) = state.team_repo.get_by_ip(&ip).await?
    {
        headers.push((HeaderName::from_static("x-wallpaper-text"), team.name));
    }

    Ok((AppendHeaders(headers), Body::from_stream(body_stream)))
}

#[derive(Serialize, ToSchema)]
pub struct ContestResponse {
    pub id: String,
    pub name: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: DateTime<Utc>,
}

impl From<Contest> for ContestResponse {
    fn from(contest: Contest) -> Self {
        Self {
            id: contest.id,
            name: contest.name,
            start_time: contest.start_time,
            end_time: contest.end_time,
        }
    }
}

#[utoipa::path(
    get,
    path = "/next-contest",
    tag = "station",
    responses(
        (status = 200, body = ContestResponse),
        (status = 404, description = "No contest ends in the future", body = ErrorResponse),
    )
)]
pub async fn next_contest(
    State(state): State<HttpHandlerState>,
) -> Result<impl IntoResponse, AppError> {
    let contest = state
        .contest_repo
        .get_next_contest()
        .await?
        .ok_or_else(|| AppError::NotFound("no upcoming contest found".to_string()))?;

    Ok(Json(ContestResponse::from(contest)))
}

#[derive(Serialize, ToSchema)]
pub struct TeamInfo {
    pub id: String,
    pub name: String,
    pub ip: String,
}

#[utoipa::path(
    get,
    path = "/team-info/{ip}",
    tag = "station",
    params(("ip" = String, Path, description = "Station address of the team")),
    responses(
        (status = 200, body = TeamInfo),
        (status = 404, description = "No team is seated at that address", body = ErrorResponse),
    )
)]
pub async fn team_info(
    State(state): State<HttpHandlerState>,
    Path(ip): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let team = state
        .team_repo
        .get_by_ip(&ip)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("team with ip {ip} not found")))?;

    Ok(Json(TeamInfo {
        id: team.id,
        name: team.name,
        ip,
    }))
}

#[utoipa::path(
    get,
    path = "/inventory",
    tag = "station",
    responses((status = 200, description = "Addresses of currently connected stations", body = Vec<String>))
)]
pub async fn station_inventory(
    State(state): State<HttpHandlerState>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(
        state
            .orchestrator
            .get_state()
            .into_iter()
            .filter(|entry| entry.connected)
            .map(|entry| entry.ip)
            .collect::<Vec<_>>(),
    ))
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct MapImageParams {
    /// Station address whose seat is highlighted.
    pub ip: Option<String>,
    /// Team whose station address is highlighted; ignored when ip is set.
    pub team_id: Option<String>,
    /// Defaults to the next upcoming contest.
    pub contest_id: Option<String>,
}

#[utoipa::path(
    get,
    path = "/map-image",
    tag = "map",
    params(MapImageParams),
    responses(
        (status = 200, description = "Rendered seating map", content_type = "image/png"),
        (status = 404, description = "No upcoming contest, or no map for the contest", body = ErrorResponse),
    )
)]
pub async fn map_image(
    State(state): State<HttpHandlerState>,
    Query(query): Query<MapImageParams>,
) -> Result<impl IntoResponse, AppError> {
    let contest_id = state.resolve_contest_id(query.contest_id).await?;

    let map = state
        .map_repo
        .get_by_contest(&contest_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("no map for contest {contest_id}")))?;

    let target_ip = state.resolve_ip(query.ip, query.team_id).await?;

    let highlight_seat_id = target_ip.as_deref().and_then(|ip| {
        map.elements.iter().find_map(|element| match element {
            MapElement::Seat(seat) if seat.ip.as_deref() == Some(ip) => Some(seat.id),
            _ => None,
        })
    });

    let png = render::render_map_png(&map, highlight_seat_id)?;

    Ok(([(header::CONTENT_TYPE, "image/png")], Body::from(png)))
}
