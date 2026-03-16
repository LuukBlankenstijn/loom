use std::env;
use std::net::SocketAddr;

use serde::Deserialize;

#[derive(Clone)]
pub struct Config {
    pub listen: SocketAddr,
    pub database: DatabaseConfig,
    pub icpc_api: Option<IcpcApiConfig>,
    pub auth_token: Option<String>,
}

#[derive(Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub name: String,
    pub password: String,
    pub sslmode: String,
}

#[derive(Clone)]
pub struct IcpcApiConfig {
    pub base_url: String,
    pub username: String,
    pub password: String,
}

// ── TOML deserialization types ───────────────────────────────────

#[derive(Deserialize, Default)]
#[serde(default)]
struct TomlConfig {
    listen: Option<SocketAddr>,
    database: TomlDatabase,
    contest_api: Option<TomlIcpcApi>,
    auth_token: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct TomlDatabase {
    host: Option<String>,
    port: Option<u16>,
    user: Option<String>,
    name: Option<String>,
    password: Option<String>,
    sslmode: Option<String>,
}

#[derive(Deserialize)]
struct TomlIcpcApi {
    base_url: String,
    username: String,
    password: String,
}

impl Config {
    pub fn load() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let path = args.get(1).map(|s| s.as_str());

        let from_file = path
            .and_then(Self::from_file)
            .or_else(|| Self::from_file("/etc/loom/backend.toml"));

        from_file.unwrap_or_else(|| {
            tracing::info!("no config file found, falling back to environment variables");
            Self::from_env()
        })
    }

    fn from_file(path: &str) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        let toml: TomlConfig = toml::from_str(&content)
            .unwrap_or_else(|e| panic!("failed to parse config file {path}: {e}"));

        tracing::info!(path, "loaded config file");

        Some(Self {
            listen: toml
                .listen
                .unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], 8080))),
            database: DatabaseConfig {
                host: toml.database.host.unwrap_or_else(|| "localhost".into()),
                port: toml.database.port.unwrap_or(5432),
                user: toml.database.user.unwrap_or_else(|| "loom".into()),
                name: toml.database.name.unwrap_or_else(|| "loom".into()),
                password: toml.database.password.unwrap_or_else(|| "loom".into()),
                sslmode: toml.database.sslmode.unwrap_or_else(|| "disable".into()),
            },
            icpc_api: toml.contest_api.map(|a| IcpcApiConfig {
                base_url: a.base_url,
                username: a.username,
                password: a.password,
            }),
            auth_token: toml.auth_token,
        })
    }

    pub fn from_env() -> Self {
        let icpc_api = env::var("DJ_BASE_URL").ok().map(|base_url| IcpcApiConfig {
            base_url,
            username: env::var("DJ_USERNAME")
                .expect("DJ_USERNAME required when DJ_BASE_URL is set"),
            password: env::var("DJ_PASSWORD")
                .expect("DJ_PASSWORD required when DJ_BASE_URL is set"),
        });

        Self {
            listen: SocketAddr::from(([0, 0, 0, 0], 8080)),
            database: DatabaseConfig {
                host: env::var("DB_HOST").expect("DB_HOST is required"),
                port: env::var("DB_PORT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(5432),
                user: env::var("DB_USER").unwrap_or_else(|_| "loom".into()),
                name: env::var("DB_DATABASE").unwrap_or_else(|_| "loom".into()),
                password: env::var("DB_PASSWORD").unwrap_or_else(|_| "loom".into()),
                sslmode: env::var("DB_SSLMODE").unwrap_or_else(|_| "disable".into()),
            },
            icpc_api,
            auth_token: env::var("AUTH_TOKEN").ok(),
        }
    }

    pub fn database_url(&self) -> String {
        self.database.database_url()
    }
}

impl DatabaseConfig {
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            self.user, self.password, self.host, self.port, self.name, self.sslmode,
        )
    }
}
