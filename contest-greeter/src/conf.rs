use std::fs;

use anyhow::Result;
use serde::Deserialize;

use crate::{greeter::GreeterConfig, ui::UiConfig};

#[derive(Debug, Clone, Deserialize)]
pub struct Conf {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub greeter: GreeterConfig,
}

fn default_log_level() -> String {
    "info".into()
}

pub fn get_conf(path: &str) -> Result<Conf> {
    let text = fs::read_to_string(path)?;
    let conf: Conf = toml::from_str(&text)?;
    Ok(conf)
}
