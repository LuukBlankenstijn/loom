mod bus;
mod greeter;
mod ui;

use env_logger::Env;
use lightdm_contest_rs_greeter::SystemHandle;
use log::{error, info};
use tokio::sync::mpsc;

use ui::run_ui;

use crate::{bus::start_bus, greeter::Greeter};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

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
        let greeter = match Greeter::new() {
            Ok(greeter) => greeter,
            Err(e) => {
                error!("[Main] failed to spawn greeter: {e}");
                return;
            }
        };
        rt.block_on(greeter.run(greeter_bus));
    });

    run_ui(bus.clone()).await;

    info!("[Main] Greeter exiting");
}
