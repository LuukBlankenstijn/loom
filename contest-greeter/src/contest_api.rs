use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use log::{debug, info};
use reqwest::Client;
use serde::Deserialize;
use tokio::time::{Duration, sleep};
use types::{CoreName, GreeterMessage, SystemSender};

#[derive(Clone, Debug, Deserialize, Default)]
pub struct ApiPollerConfig {
    #[serde(default = "default_interval")]
    interval: i64,

    url: Option<String>,
}

fn default_interval() -> i64 {
    3
}

pub async fn run_api_poller(bus: impl SystemSender, config: ApiPollerConfig) {
    let Some(url) = config.url else {
        info!("[Contest-Api] contest url not set, not running api poller");
        return;
    };

    // keep the bus alive for future message routing even though it is unused for now
    let poll_interval = Duration::from_secs(config.interval.max(0) as u64);

    let client = Client::new();

    loop {
        match fetch_start_time(&client, &url).await {
            Ok(start_time) => {
                if start_time < Local::now() {
                    info!("[Contest-Api] contest started at {start_time}");
                    bus.send_to(CoreName::Greeter, GreeterMessage::Login());
                } else {
                    debug!("[Contest-Api] contest not started yet (starts at {start_time})");
                }
            }
            Err(e) => debug!("[Contest-Api] failed to poll contest API ({url}): {:#}", e),
        }

        sleep(poll_interval).await;
    }
}

#[derive(Deserialize)]
struct ContestApiResponse {
    start_time: String,
}

async fn fetch_start_time(client: &Client, url: &str) -> Result<DateTime<Local>> {
    let response = client
        .get(url)
        .send()
        .await
        .context(format!("sending request to {url}"))?
        .error_for_status()
        .context("server returned error")?;

    let payload: ContestApiResponse = response.json().await.context("decoding JSON payload")?;

    let parsed = DateTime::parse_from_rfc3339(&payload.start_time)
        .context("parsing start_time as RFC3339 timestamp")?;

    Ok(parsed.with_timezone(&Local))
}
