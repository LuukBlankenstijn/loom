use tracing::Level;
use tracing_subscriber::{filter::Targets, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;

mod api;
mod config;
mod domain;
mod error;
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

    let repositories = repository::Repositories::new(config.database, config.icpc_api).await;
    let contest_repo = repositories.get_contest();
    let map_repo = repositories.get_map();
    let station_repo = repositories.get_station();
    let team_repo = repositories.get_team();

    let contest_handler = api::ContestHandler::new(contest_repo.clone(), map_repo.clone());
    let station_handler = api::StationHandler::new(
        contest_repo.clone(),
        station_repo.clone(),
        team_repo.clone(),
    );
    let team_handler = api::TeamHandler::new(contest_repo.clone(), team_repo.clone());
    let map_handler = api::MapHandler::new(map_repo);

    Ok(())
}
