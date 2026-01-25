use chrono::{DateTime, Utc};

use anyhow::Result;
use async_trait::async_trait;
use std::fmt::Debug;

#[async_trait]
pub trait AdminService: Send + Sync + Debug {
    async fn fetch_stations(&self) -> Result<Vec<Station>>;
    async fn fetch_teams(&self) -> Result<Vec<Team>>;
    async fn fetch_contest(&self) -> Result<Option<Contest>>;

    async fn set_ip(&self, team_id: String, ip: Option<String>) -> Result<()>;
    async fn set_wallpaper(&self, contest_id: String, image: Option<Vec<u8>>) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct Station {
    pub id: i32,
    pub ip: String,
    pub connected_at: DateTime<Utc>,
    pub disconnected_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Team {
    pub id: String,
    pub ip: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Contest {
    pub id: String,
    pub name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}
