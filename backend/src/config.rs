use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use serde::Deserialize;

const DEFAULT_CONFIG_PATH: &str = "/etc/loom/backend.toml";

#[derive(Clone)]
pub struct Config {
    pub listen: SocketAddr,
    pub database_path: PathBuf,
    pub dashboard_dir: Option<PathBuf>,
    pub icpc_api: Option<IcpcApiConfig>,
    pub auth_token: Option<String>,
}

#[derive(Clone)]
pub struct IcpcApiConfig {
    pub base_url: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TomlConfig {
    #[serde(default = "default_listen")]
    listen: SocketAddr,
    database_path: PathBuf,
    dashboard_dir: Option<PathBuf>,
    auth_token: Option<String>,
    auth_token_file: Option<PathBuf>,
    contest_api: Option<TomlIcpcApi>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TomlIcpcApi {
    base_url: String,
    username: String,
    password: Option<String>,
    password_file: Option<PathBuf>,
}

fn default_listen() -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], 8080))
}

fn read_secret(
    key: &str,
    value: Option<String>,
    file: Option<PathBuf>,
) -> Result<Option<String>, String> {
    match (value, file) {
        (Some(_), Some(_)) => Err(format!("set either {key} or {key}_file, not both")),
        (value, None) => Ok(value),
        (None, Some(file)) => std::fs::read_to_string(&file)
            .map(|secret| Some(secret.trim_end().to_owned()))
            .map_err(|e| format!("failed to read {key}_file {}: {e}", file.display())),
    }
}

impl Config {
    pub fn load() -> Result<Self, String> {
        if let Some(path) = env::args().nth(1) {
            return Self::from_file(Path::new(&path));
        }

        let default_path = Path::new(DEFAULT_CONFIG_PATH);
        if default_path.exists() {
            return Self::from_file(default_path);
        }

        tracing::info!("no config file found, falling back to environment variables");
        Self::from_env()
    }

    fn from_file(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read config file {}: {e}", path.display()))?;
        let toml: TomlConfig = toml::from_str(&content)
            .map_err(|e| format!("failed to parse config file {}: {e}", path.display()))?;

        tracing::info!(path = %path.display(), "loaded config file");

        let icpc_api = toml
            .contest_api
            .map(|api| {
                let password = read_secret("password", api.password, api.password_file)?
                    .ok_or("contest_api requires password or password_file")?;
                Ok::<_, String>(IcpcApiConfig {
                    base_url: api.base_url,
                    username: api.username,
                    password,
                })
            })
            .transpose()?;

        Ok(Self {
            listen: toml.listen,
            database_path: toml.database_path,
            dashboard_dir: toml.dashboard_dir,
            icpc_api,
            auth_token: read_secret("auth_token", toml.auth_token, toml.auth_token_file)?,
        })
    }

    fn from_env() -> Result<Self, String> {
        let icpc_api = env::var("DJ_BASE_URL")
            .ok()
            .map(|base_url| {
                Ok::<_, String>(IcpcApiConfig {
                    base_url,
                    username: env::var("DJ_USERNAME")
                        .map_err(|_| "DJ_USERNAME required when DJ_BASE_URL is set")?,
                    password: env::var("DJ_PASSWORD")
                        .map_err(|_| "DJ_PASSWORD required when DJ_BASE_URL is set")?,
                })
            })
            .transpose()?;

        Ok(Self {
            listen: default_listen(),
            database_path: env::var_os("DATABASE_PATH")
                .ok_or("DATABASE_PATH is required")?
                .into(),
            dashboard_dir: env::var_os("DASHBOARD_DIR").map(PathBuf::from),
            icpc_api,
            auth_token: env::var("AUTH_TOKEN").ok(),
        })
    }
}
