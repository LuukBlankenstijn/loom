use tracing::Level;
use tracing_subscriber::{filter::Targets, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;

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

    let repositories = repository::RepoContainer::new(config.database, config.icpc_api).await;

    Ok(())
}
