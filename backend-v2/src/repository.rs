use std::sync::Arc;

use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::{
    config::{DatabaseConfig, IcpcApiConfig},
    domain::{ContestRepository, MapRepository, StationRepository, TeamRepository},
};

mod contest;
mod map;
mod station;
mod team;

mod utils;

#[derive(Clone)]
pub struct Repositories {
    pool: PgPool,
    http_client: reqwest::Client,

    icpc_config: Option<IcpcApiConfig>,
}

impl Repositories {
    pub async fn new(
        database_config: DatabaseConfig,
        icpc_config: Option<IcpcApiConfig>,
    ) -> Repositories {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(100))
            .build()
            .expect("failed to build http client");

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_config.database_url())
            .await
            .expect("Failed to build pgPool");

        Self {
            http_client,
            pool,
            icpc_config,
        }
    }
    pub fn get_contest(&self) -> Arc<dyn ContestRepository> {
        Arc::new(contest::ContestRepo::new(
            self.pool.clone(),
            self.http_client.clone(),
            self.icpc_config.clone(),
        ))
    }

    pub fn get_team(&self) -> Arc<dyn TeamRepository> {
        Arc::new(team::TeamRepo::new(
            self.pool.clone(),
            self.http_client.clone(),
            self.icpc_config.clone(),
        ))
    }

    pub fn get_map(&self) -> Arc<dyn MapRepository> {
        Arc::new(map::MapRepo::new(self.pool.clone()))
    }

    pub fn get_station(&self) -> Arc<dyn StationRepository> {
        Arc::new(station::StationRepo::new(self.pool.clone()))
    }
}
