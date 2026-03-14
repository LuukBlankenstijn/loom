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

pub struct RepoContainer {
    pool: PgPool,
    http_client: reqwest::Client,

    icpc_config: Option<IcpcApiConfig>,
}

impl RepoContainer {
    pub async fn new(database_config: DatabaseConfig, icpc_config: Option<IcpcApiConfig>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
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

    pub fn get_contest(&self) -> impl ContestRepository {
        contest::ContestRepo::new(
            self.pool.clone(),
            self.http_client.clone(),
            self.icpc_config.clone(),
        )
    }

    pub fn get_team(&self) -> impl TeamRepository {
        team::TeamRepo::new(
            self.pool.clone(),
            self.http_client.clone(),
            self.icpc_config.clone(),
        )
    }

    pub fn get_map(&self) -> impl MapRepository {
        map::MapRepo::new(self.pool.clone())
    }

    pub fn get_station(&self) -> impl StationRepository {
        station::StationRepo::new(self.pool.clone())
    }
}
