use crate::conf::{Conf, get_conf};
use env_logger::Env;
use log::warn;
use std::env;
mod conf;
mod ipc;
mod subscriptions;
mod ui;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config_path = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("/etc/greetd/contest-greeter.toml");

    let config = match get_conf(config_path) {
        Ok(config) => config,
        Err(e) => {
            warn!(
                "failed to find config at {}, using default config: {e}",
                config_path
            );
            Conf::new_default()
        }
    };

    env_logger::Builder::from_env(
        Env::default().default_filter_or(format!("contest_greeter={}", config.log_level)),
    )
    .init();

    if let Err(e) = ui::run_greeter(config) {
        println!("error running app: {:?}", e)
    }
}
