mod api;
mod config;
mod convert;
mod domain;
mod error;
mod hub;
mod repo;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::get;
use axum::{Router, middleware};
use loom_rpc::admin::v1::admin_service_server::AdminServiceServer;
use loom_rpc::stations::v1::station_service_server::StationServiceServer;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

use config::Config;
use hub::StationsHub;
use tracing::Level;
use tracing_subscriber::{filter::Targets, fmt, prelude::*};

use crate::convert::CommandOutput;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let filter = Targets::new()
        .with_target("loom_backend", Level::TRACE)
        .with_default(Level::WARN);

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    let config = Config::load();

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url())
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    // Wire repositories
    let station_repo: Arc<dyn domain::StationRepository> =
        Arc::new(repo::PgStationRepo::new(pool.clone()));
    let wallpaper_repo: Arc<dyn domain::WallpaperRepository> =
        Arc::new(repo::PgWallpaperRepo::new(pool.clone()));
    let map_repo: Arc<dyn domain::MapRepository> = Arc::new(repo::PgMapRepo::new(pool.clone()));

    let (contest_repo, team_repo): (
        Arc<dyn domain::ContestRepository>,
        Arc<dyn domain::TeamRepository>,
    ) = if let Some(icpc) = &config.icpc_api {
        (
            Arc::new(repo::HttpContestRepo::new(icpc.clone())),
            Arc::new(repo::HttpTeamRepo::new(icpc.clone())),
        )
    } else {
        (
            Arc::new(repo::PgContestRepo::new(pool.clone())),
            Arc::new(repo::PgTeamRepo::new(pool.clone())),
        )
    };

    let hub = StationsHub::new();

    let (client_broadcast, _) = broadcast::channel::<CommandOutput>(32);

    // gRPC services
    let admin = api::admin::AdminHandler::new(
        contest_repo.clone(),
        team_repo.clone(),
        station_repo.clone(),
        wallpaper_repo.clone(),
        map_repo.clone(),
        hub.clone(),
        client_broadcast.clone(),
    );

    let stations = api::stations::StationsHandler::new(
        hub.clone(),
        contest_repo.clone(),
        station_repo.clone(),
        config.icpc_api.as_ref().map(|c| c.base_url.clone()),
        client_broadcast.clone(),
    );

    // Wallpaper HTTP handler state
    let wallpaper_state = Arc::new(api::wallpaper::WallpaperState::new(
        contest_repo,
        team_repo,
        wallpaper_repo,
    ));

    let grpc_router = tonic::service::Routes::new(StationServiceServer::new(stations))
        .add_service(AdminServiceServer::new(admin))
        .into_axum_router()
        .layer(tonic_web::GrpcWebLayer::new())
        .layer(middleware::from_fn(
            api::middleware::client_meta_interceptor,
        ));

    // Merge gRPC with axum HTTP routes
    let app = Router::new()
        .route("/wallpaper", get(api::wallpaper::wallpaper_handler))
        .with_state(wallpaper_state)
        .merge(grpc_router)
        .layer(CorsLayer::permissive());

    tracing::info!(addr = %config.listen, "starting server");

    let listener = tokio::net::TcpListener::bind(config.listen).await?;
    let service = app.into_make_service_with_connect_info::<SocketAddr>();
    axum::serve(listener, service).await?;

    Ok(())
}
