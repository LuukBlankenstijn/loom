use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset, Local, TimeDelta};
use iced::{Subscription, Task, time};
use log::{debug, error};
use serde::Deserialize;

use crate::ui::Message;

#[derive(Debug)]
pub struct ApiPoller {
    url: Option<String>,
    start_time: Option<DateTime<Local>>,
    interval: u64,
}

#[derive(Clone, Debug)]
pub enum ApiPollerMessage {
    Tick,
    StartTimeFetched(Result<Option<DateTime<Local>>, String>),
    SetUrl(Option<String>),
    SetStarttime(Option<DateTime<Local>>),
}

impl From<ApiPollerMessage> for Message {
    fn from(value: ApiPollerMessage) -> Self {
        Message::ApiPoller(value)
    }
}

impl ApiPoller {
    pub fn new(url: Option<String>) -> (Self, Task<ApiPollerMessage>) {
        (
            Self {
                url,
                start_time: None,
                interval: 60,
            },
            Task::done(ApiPollerMessage::Tick),
        )
    }

    pub fn update(&mut self, msg: ApiPollerMessage) -> Task<ApiPollerMessage> {
        match msg {
            ApiPollerMessage::Tick => {
                if let Some(url) = self.url.clone() {
                    return Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                fetch_start_time(&url).map_err(|e| e.to_string())
                            })
                            .await
                            .unwrap_or_else(|_| Err("Task panicked".to_string()))
                        },
                        ApiPollerMessage::StartTimeFetched,
                    );
                }
                return Task::none();
            }
            ApiPollerMessage::SetUrl(url) => {
                self.url = url;
                return Task::done(ApiPollerMessage::Tick);
            }
            ApiPollerMessage::StartTimeFetched(result) => match result {
                Ok(fetched_start_time) => {
                    if let Some(fetched) = fetched_start_time {
                        self.interval = if fetched - Local::now() < TimeDelta::minutes(2) {
                            5
                        } else {
                            60
                        };
                    }

                    if self.start_time == fetched_start_time {
                        return Task::none();
                    }
                    self.start_time = fetched_start_time;
                    return Task::done(ApiPollerMessage::SetStarttime(fetched_start_time));
                }
                Err(error) => error!("failed getting starttime from api:{error}"),
            },
            _ => {}
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<ApiPollerMessage> {
        time::every(Duration::from_secs(self.interval)).map(|_| ApiPollerMessage::Tick)
    }
}

#[derive(Deserialize)]
struct ContestApiResponse {
    #[serde(default)]
    start_time: Option<DateTime<FixedOffset>>,
}

fn fetch_start_time(url: &str) -> Result<Option<DateTime<Local>>> {
    debug!("fetch start time from {url}");
    let mut response = ureq::get(url)
        .call()
        .context(format!("sending request to {url}"))?;

    let payload: ContestApiResponse = response
        .body_mut()
        .read_json()
        .context("decoding JSON payload")?;

    Ok(payload.start_time.map(|t| t.with_timezone(&Local)))
}
