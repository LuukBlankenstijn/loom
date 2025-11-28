mod core;
mod lightdm;
mod ui;

use env_logger::Env;
use log::info;
use tokio::signal;

use lightdm::Greeter;

use crate::ui::run_ui;

#[tokio::main]
async fn main() {
    info!("Rust greeter starting up…");
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    match Greeter::new() {
        Ok(greeter) => {
            if let Err(e) = greeter.connect_to_daemon() {
                eprintln!("Failed to connect to LightDM daemon: {e}");
            }

            let default_session = greeter
                .default_session_hint()
                .unwrap_or_else(|| "<none>".to_string());
            let autologin_user = greeter
                .autologin_user_hint()
                .unwrap_or_else(|| "<none>".to_string());

            info!("Connected to LightDM daemon");
            info!("  default_session_hint: {default_session}");
            info!("  autologin_user_hint : {autologin_user}");
        }
        Err(e) => {
            eprintln!("failed to construct greeter: {e}");
        }
    };

    run_ui();

    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received SIGINT, shutting down greeter");
        }
    }

    info!("Rust greeter exiting");
}
