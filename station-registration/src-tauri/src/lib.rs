use anyhow::{Context, Result};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{Manager, State};
use tauri_plugin_cli::CliExt;

#[derive(Default)]
struct CliOverride {
    server: Mutex<Option<String>>,
    auth: Mutex<Option<String>>,
    ip: Mutex<Option<String>>,
}

#[derive(Serialize)]
struct StationConfig {
    server: String,
    auth: Option<String>,
    ip: String,
}

#[tauri::command]
fn get_station_config(cli: State<'_, CliOverride>) -> Result<StationConfig, String> {
    let server = cli
        .server
        .lock()
        .unwrap()
        .clone()
        .or_else(|| std::env::var("LOOM_SERVER").ok())
        .unwrap_or_else(|| "http://localhost:8080".to_string());

    let auth = cli
        .auth
        .lock()
        .unwrap()
        .clone()
        .or_else(|| std::env::var("LOOM_AUTH").ok());

    let ip = cli
        .ip
        .lock()
        .unwrap()
        .clone()
        .map(Ok)
        .unwrap_or_else(detect_ip)
        .map_err(|e| e.to_string())?;

    Ok(StationConfig { server, auth, ip })
}

fn detect_ip() -> Result<String> {
    let ip = local_ip_address::local_ip().context("failed to detect local network IP")?;
    Ok(ip.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_cli::init())
        .manage(CliOverride::default())
        .setup(|app| {
            if let Ok(matches) = app.cli().matches() {
                let cli = app.state::<CliOverride>();
                if let Some(arg) = matches.args.get("server")
                    && let Some(s) = arg.value.as_str()
                {
                    *cli.server.lock().unwrap() = Some(s.to_string());
                }
                if let Some(arg) = matches.args.get("auth")
                    && let Some(s) = arg.value.as_str()
                {
                    *cli.auth.lock().unwrap() = Some(s.to_string());
                }
                if let Some(arg) = matches.args.get("ip")
                    && let Some(s) = arg.value.as_str()
                {
                    *cli.ip.lock().unwrap() = Some(s.to_string());
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_station_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
