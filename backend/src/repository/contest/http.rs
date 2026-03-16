use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::InnerRepo;
use crate::config::IcpcApiConfig;
use crate::domain::Contest;
use crate::error::AppError;

pub struct HttpContestRepo {
    client: reqwest::Client,
    config: IcpcApiConfig,
}

#[derive(Deserialize)]
struct ContestDto {
    id: String,
    name: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
}

impl HttpContestRepo {
    pub fn new(config: IcpcApiConfig, client: reqwest::Client) -> Self {
        Self { client, config }
    }
}
#[async_trait]
impl InnerRepo for HttpContestRepo {
    async fn get_next_contest(&self) -> Result<Option<Contest>, AppError> {
        let url = format!("{}/api/v4/contests", self.config.base_url);
        let body = self
            .client
            .get(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await?
            .error_for_status()
            .map_err(|e| AppError::Internal(format!("contest API error: {e}")))?
            .text()
            .await?;

        let contests: Vec<ContestDto> = serde_json::from_str(&body).map_err(|e| {
            tracing::error!(url, body, error = ?e, "failed to decode contest API response");
            AppError::Internal(format!("contest API decode error: {e}"))
        })?;

        let now = Utc::now();
        let mut upcoming: Vec<_> = contests.into_iter().filter(|c| c.end_time > now).collect();
        upcoming.sort_by_key(|c| c.start_time);

        Ok(upcoming.into_iter().next().map(|c| Contest {
            id: c.id,
            name: c.name,
            start_time: c.start_time,
            end_time: c.end_time,
        }))
    }
}
