use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset, Local};
use iced::{Subscription, Task, time};
use log::{debug, error};
use serde::Deserialize;

use crate::ui::Message;

#[derive(Debug)]
pub struct ApiPoller {
    url: Option<String>,
}

#[derive(Clone, Debug)]
pub enum ApiPollerMessage {
    FetchStartTime,
    StartTimeFetched(Result<DateTime<Local>, String>),
    SetUrl(Option<String>),
    SetStartime(DateTime<Local>),
}

impl From<ApiPollerMessage> for Message {
    fn from(value: ApiPollerMessage) -> Self {
        Message::ApiPoller(value)
    }
}

impl ApiPoller {
    pub fn new(url: Option<String>) -> (Self, Task<ApiPollerMessage>) {
        (Self { url }, Task::done(ApiPollerMessage::FetchStartTime))
    }

    pub fn update(&mut self, msg: ApiPollerMessage) -> Task<ApiPollerMessage> {
        match msg {
            ApiPollerMessage::FetchStartTime => {
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
            ApiPollerMessage::SetUrl(url) => self.url = url,
            ApiPollerMessage::StartTimeFetched(result) => match result {
                Ok(datetime) => return Task::done(ApiPollerMessage::SetStartime(datetime)),
                Err(error) => error!("failed getting starttime from api:{error}"),
            },
            _ => {}
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<ApiPollerMessage> {
        time::every(Duration::from_secs(60)).map(|_| ApiPollerMessage::FetchStartTime)
    }
}

#[derive(Deserialize)]
struct ContestApiResponse {
    start_time: DateTime<FixedOffset>,
}

fn fetch_start_time(url: &str) -> Result<DateTime<Local>> {
    debug!("fetch start time from {url}");
    let mut response = ureq::get(url)
        .call()
        .context(format!("sending request to {url}"))?;

    let payload: ContestApiResponse = response
        .body_mut()
        .read_json()
        .context("decoding JSON payload")?;

    Ok(payload.start_time.with_timezone(&Local))
}
