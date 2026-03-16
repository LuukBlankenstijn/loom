use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    config::IcpcApiConfig,
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
    pub async fn new(pool: PgPool, icpc_config: Option<IcpcApiConfig>) -> Repositories {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(100))
            .build()
            .expect("failed to build http client");

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
