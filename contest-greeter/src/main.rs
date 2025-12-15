mod bus;
mod conf;
mod contest_api;
mod greeter;
mod ui;

use env_logger::Env;
use lightdm_contest_rs_greeter::SystemHandle;
use log::{error, info};
use tokio::sync::mpsc;

use ui::run_ui;

use crate::{
    bus::start_bus, conf::get_conf, contest_api::run_api_poller, greeter::Greeter,
};

#[tokio::main]
async fn main() {
    let config = match get_conf("/etc/lightdm/lightdm-contest-greeter.conf") {
        Ok(config) => config,
        Err(e) => panic!("failed to read config: {e}"),
    };

    env_logger::Builder::from_env(Env::default().default_filter_or(config.log_level)).init();

    let (bus_tx, bus_rx) = mpsc::channel(16);
    let bus = SystemHandle::new(bus_tx);

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(start_bus(bus_rx));
    });

    let greeter_bus = bus.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        let greeter = match Greeter::new(config.greeter) {
            Ok(greeter) => greeter,
            Err(e) => {
                error!("[Main] failed to spawn greeter: {e}");
                return;
            }
        };
        rt.block_on(greeter.run(greeter_bus));
    });

    let api_bus = bus.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(run_api_poller(api_bus, config.api_poller));
    });

    run_ui(bus.clone(), config.ui).await;

    info!("[Main] Greeter exiting");
}
