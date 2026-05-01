use std::sync::Arc;

use axum::{Router, routing::get};
use loom_rpc::{
    admin::v1::{
        contest_service_server::ContestServiceServer, station_service_server::StationServiceServer,
        team_service_server::TeamServiceServer,
    },
    broadcast::v1::broadcast_service_server::BroadcastServiceServer,
    map::v1::map_service_server::MapServiceServer,
    station::v1::station_service_server,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tracing::Level;
use tracing_subscriber::{filter::Targets, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{api::combined_auth_interceptor, config::Config, orchestrator::Orchestrator};

mod api;
mod config;
mod domain;
mod error;
mod orchestrator;
mod repository;

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
        .max_connections(100)
        .connect(&config.database_url())
        .await
        .expect("Failed to build pgPool");

    sqlx::migrate!("./migrations").run(&pool).await?;

    let repositories = repository::Repositories::new(pool, config.icpc_api.clone()).await;
    let contest_repo = repositories.get_contest();
    let map_repo = repositories.get_map();
    let station_repo = repositories.get_station();
    let team_repo = repositories.get_team();

    let orchestrator: Arc<dyn domain::Orchestrator> = Arc::new(Orchestrator::new());

    let interceptor = combined_auth_interceptor(config.auth_token);

    let contest_service = ContestServiceServer::with_interceptor(
        api::admin::ContestHandler::new(
            contest_repo.clone(),
            map_repo.clone(),
            orchestrator.clone(),
        ),
        interceptor.clone(),
    );
    let station_service = StationServiceServer::with_interceptor(
        api::admin::StationHandler::new(
            contest_repo.clone(),
            station_repo.clone(),
            team_repo.clone(),
            map_repo.clone(),
            orchestrator.clone(),
        ),
        interceptor.clone(),
    );
    let team_service = TeamServiceServer::with_interceptor(
        api::admin::TeamHandler::new(
            contest_repo.clone(),
            team_repo.clone(),
            map_repo.clone(),
            orchestrator.clone(),
        ),
        interceptor.clone(),
    );
    let map_service = MapServiceServer::with_interceptor(
        api::map::MapHandler::new(
            orchestrator.clone(),
            map_repo.clone(),
            team_repo.clone(),
        ),
        interceptor.clone(),
    );

    let broadcast_service = BroadcastServiceServer::with_interceptor(
        api::broadcast::BroadcastHandler::new(
            orchestrator.clone(),
            map_repo.clone(),
            team_repo.clone(),
            contest_repo.clone(),
        ),
        interceptor.clone(),
    );
    let station_stream_service = station_service_server::StationServiceServer::with_interceptor(
        api::station::StationHandler::new(station_repo.clone(), orchestrator.clone()),
        interceptor.clone(),
    );

    let wallpaper_handler = api::http::HttpHandlerState::new(contest_repo, team_repo);

    let grpc_router = tonic::service::Routes::builder()
        .add_service(contest_service)
        .add_service(station_service)
        .add_service(team_service)
        .add_service(map_service)
        .add_service(broadcast_service)
        .add_service(station_stream_service)
        .clone()
        .routes()
        .into_axum_router()
        .layer(tonic_web::GrpcWebLayer::new());

    let routes = Router::new()
        .route("/wallpaper", get(api::http::wallpaper_handler))
        .route("/next-contest", get(api::http::next_contest))
        .route("/team-info/{ip}", get(api::http::team_info))
        .with_state(wallpaper_handler)
        .merge(grpc_router)
        .layer(CorsLayer::permissive());

    tracing::info!(addr = %config.listen, "starting server");

    let listener = tokio::net::TcpListener::bind(config.listen).await?;
    let service = routes.into_make_service();
    axum::serve(listener, service).await?;
    Ok(())
}
