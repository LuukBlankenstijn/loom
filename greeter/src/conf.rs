use std::{fs, process::Command};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Secret {
    Static(String),
    Command { command: String },
}

impl Default for Secret {
    fn default() -> Self {
        Self::Static(String::new())
    }
}

impl From<Secret> for String {
    fn from(value: Secret) -> Self {
        match value {
            Secret::Static(s) => s,
            Secret::Command { command } => {
                let output = Command::new("sh").arg("-c").arg(&command).output();

                match output {
                    Ok(out) if out.status.success() => {
                        String::from_utf8_lossy(&out.stdout).trim().to_string()
                    }
                    Ok(out) => {
                        let err = String::from_utf8_lossy(&out.stderr);
                        panic!("Config secret command '{}' failed: {}", command, err)
                    }
                    Err(e) => panic!("Failed to get secret'{}': {}", command, e),
                }
            }
        }
    }
}

fn deserialize_secret<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let s = Secret::deserialize(deserializer)?;
    Ok(String::from(s))
}

/// Top-level configuration combining UI, greeter, and contest API settings.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Conf {
    /// Log level (env_logger style, e.g. `info`, `debug`).
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Enable or disable the dbus module
    #[serde(default = "default_enable_dbus")]
    pub enable_dbus: bool,

    /// Key sequence to toggle the login UI.
    #[serde(default = "default_chain", deserialize_with = "deserialize_secret")]
    pub(crate) chain: String,

    /// File path or URL for the background image.
    pub(crate) background_source: Option<String>,

    // Label to display over the background
    pub(crate) background_label: Option<String>,

    // Hex code of the color to use for the background label
    pub(crate) background_label_color: Option<String>,

    /// Session to start (defaults to LightDM's default when unset).
    pub(crate) session: Option<String>,

    /// Username used for automatic login.
    #[serde(default)]
    pub(crate) username: String,

    /// Password used for automatic login.
    #[serde(default, deserialize_with = "deserialize_secret")]
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
