use std::fs;

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Top-level configuration combining UI, greeter, and contest API settings.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct Conf {
    /// Log level (env_logger style, e.g. `info`, `debug`).
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Enable or disable the dbus module
    #[serde(default = "default_enable_dbus")]
    pub enable_dbus: bool,

    /// Key sequence to toggle the login UI.
    #[serde(default = "default_chain")]
    pub(crate) chain: String,

    /// File path or URL for the background image.
    pub(crate) background_source: Option<String>,

    /// Session to start (defaults to LightDM's default when unset).
    pub(crate) session: Option<String>,

    /// Username used for automatic login.
    #[serde(default)]
    pub(crate) username: String,

    /// Password used for automatic login.
    #[serde(default)]
    pub(crate) password: String,

    /// Contest API URL returning a JSON object with `start_time` (RFC3339).
    pub(crate) url: Option<String>,
}

fn default_log_level() -> String {
    "info".into()
}

fn default_enable_dbus() -> bool {
    true
}

fn default_chain() -> String {
    "chain".into()
}

impl Conf {
    pub fn new_default() -> Self {
        Self {
            log_level: default_log_level(),
            enable_dbus: default_enable_dbus(),
            chain: default_chain(),
            ..Default::default()
        }
    }
}

pub fn get_conf(path: &str) -> Result<Conf> {
    let text = fs::read_to_string(path)?;
    let conf: Conf = toml::from_str(&text)?;
    Ok(conf)
}
