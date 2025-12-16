use std::fs;

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{contest_api::ApiPollerConfig, greeter::GreeterConfig, ui::UiConfig};

/// Top-level configuration combining UI, greeter, and contest API settings.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Conf {
    /// Log level (env_logger style, e.g. `info`, `debug`).
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// UI settings.
    #[serde(flatten, default)]
    pub ui: UiConfig,

    /// Greeter settings.
    #[serde(flatten, default)]
    pub greeter: GreeterConfig,

    /// Contest API poller settings.
    #[serde(flatten, default)]
    pub api_poller: ApiPollerConfig,
}

fn default_log_level() -> String {
    "info".into()
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            ui: UiConfig::default(),
            greeter: GreeterConfig::default(),
            api_poller: ApiPollerConfig::default(),
        }
    }
}

pub fn get_conf(path: &str) -> Result<Conf> {
    let text = fs::read_to_string(path)?;
    let conf: Conf = toml::from_str(&text)?;
    Ok(conf)
}
