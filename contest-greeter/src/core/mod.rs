use lightdm_contest_rs_greeter::{CoreUICommand, UICoreCommand};
use log::error;
use tokio::sync::mpsc;

use crate::core::greeter::{CoreCommand, Greeter};

mod greeter;

pub async fn run_core(
    mut core_rx: mpsc::UnboundedReceiver<UICoreCommand>,
    ui_tx: mpsc::UnboundedSender<CoreUICommand>,
) {
    let (greeter_tx, mut greeter_rx) = mpsc::unbounded_channel::<CoreCommand>();
    let greeter = match Greeter::new() {
        Ok(greeter) => {
            greeter.init(greeter_tx);
            greeter
        }
        Err(e) => {
            error!("failed to create greeter: {e}");
            return;
        }
    };

    loop {
        tokio::select! {
            Some(command) = core_rx.recv() => {
                match command {
                    UICoreCommand::Login(username, password) => {
                        greeter.authenticate(username, password);
                    },
                }
            },

            Some(command) = greeter_rx.recv() => {
                match command {
                    CoreCommand::StartSession(session) => greeter.start_session(session.as_deref()),
                    CoreCommand::SetError(error) => { let _ = ui_tx.send(CoreUICommand::SetError(error)); },
                }

            }

            else => {
                break
            }
        }
    }
}
