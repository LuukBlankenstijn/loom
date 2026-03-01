use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use tracing::error;

use crate::domain::{ContestRepository, TeamRepository, WallpaperRepository};

struct CachedWallpaper {
    data: Vec<u8>,
    mime_type: String,
    text_color: String,
    updated_at: DateTime<Utc>,
}

pub struct WallpaperState {
    contest_repo: Arc<dyn ContestRepository>,
    team_repo: Arc<dyn TeamRepository>,
    wallpaper_repo: Arc<dyn WallpaperRepository>,
    cache: RwLock<Option<CachedWallpaper>>,
}

impl WallpaperState {
    pub fn new(
        contest_repo: Arc<dyn ContestRepository>,
        team_repo: Arc<dyn TeamRepository>,
        wallpaper_repo: Arc<dyn WallpaperRepository>,
    ) -> Self {
        Self {
            contest_repo,
            team_repo,
            wallpaper_repo,
            cache: RwLock::new(None),
        }
    }

    async fn update_cache(&self, contest_id: &str) -> Result<(), StatusCode> {
        let db_updated_at = self
            .wallpaper_repo
            .get_last_updated(contest_id)
            .await
            .map_err(|e| {
                error!(%e, "failed to get wallpaper last_updated");
                StatusCode::SERVICE_UNAVAILABLE
            })?;

        // Fast path: check if cache is current
        {
            let cache = self.cache.read().await;
            if let Some(cached) = &*cache
                && Some(cached.updated_at) == db_updated_at
            {
                return Ok(());
            }
        }

        // Slow path: fetch full blob under write lock
        let mut cache = self.cache.write().await;

        // Double-check after acquiring write lock
        if let Some(cached) = &*cache
            && Some(cached.updated_at) == db_updated_at
        {
            return Ok(());
        }

        let wp = self
            .wallpaper_repo
            .get_wallpaper(contest_id)
            .await
            .map_err(|e| {
                error!(%e, "failed to get wallpaper");
                StatusCode::SERVICE_UNAVAILABLE
            })?;

        *cache = wp.map(|w| CachedWallpaper {
            data: w.data,
            mime_type: w.mime_type,
            text_color: w.text_color,
            updated_at: w.updated_at,
        });

        Ok(())
    }
}

pub async fn wallpaper_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(state): State<Arc<WallpaperState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let ip = headers
        .get("x-real-ip")
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|| addr.ip().to_string());

    let (contest, team) = tokio::join!(state.contest_repo.get_next_contest(), async {
        if ip.is_empty() {
            Ok(None)
        } else {
            state.team_repo.get_by_ip(&ip).await
        }
    });

    let contest = contest
        .map_err(|e| {
            error!(%e, "failed to get next contest");
            StatusCode::NOT_FOUND
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    let team = team
        .inspect_err(|e| error!(%e, %ip, "failed to get team by ip"))
        .ok()
        .flatten();

    state.update_cache(&contest.id).await?;

    let cache = state.cache.read().await;
    let cached = cache.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let mut response_headers = HeaderMap::new();
    if let Ok(v) = cached.mime_type.parse() {
        response_headers.insert(header::CONTENT_TYPE, v);
    }
    response_headers.insert(header::CACHE_CONTROL, "no-cache".parse().unwrap());

    if let Some(team) = team {
        if let Ok(v) = team.name.parse() {
            response_headers.insert("X-Wallpaper-Text", v);
        }
        if let Ok(v) = cached.text_color.parse() {
            response_headers.insert("X-Wallpaper-Text-Color", v);
        }
    }

    Ok((response_headers, cached.data.clone()))
}
