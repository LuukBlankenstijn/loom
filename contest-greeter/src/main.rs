mod core;
mod lightdm;
mod ui;

use env_logger::Env;
use lightdm_contest_rs_greeter::{CoreUICommand, UICoreCommand};
use log::info;
use tokio::{signal, sync::mpsc};

use core::run_core;
use ui::run_ui;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    info!("Greeter starting up…");

    let (core_tx, core_rx) = mpsc::unbounded_channel::<UICoreCommand>();
    let (ui_tx, ui_rx) = mpsc::unbounded_channel::<CoreUICommand>();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(run_core(core_rx, ui_tx));
    });

    run_ui(core_tx, ui_rx);

    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received SIGINT, shutting down greeter");
        }
    }

    info!("Greeter exiting");
}
