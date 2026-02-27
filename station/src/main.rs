use clap::Parser;
use tracing::Level;
use tracing::debug;
use tracing::error;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::command::CommandRunner;
use crate::dbus::DbusClient;
use crate::messages::CommandRunnerCommand;
use crate::messages::DbusCommand;
use crate::messages::RpcCommand;
use crate::rpc::RpcClient;

mod command;
mod dbus;
mod messages;
mod rpc;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Server to connect to
    #[arg(short, long)]
    server: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();
    let args = Args::parse();
    let (dbus_client, dbus_sender, mut dbus_receiver) = DbusClient::new().await?;
    tokio::spawn(async move {
        if let Err(e) = dbus_client.run().await {
            error!("Dbus client error: {}", e);
        }
    });

    let (rpc_client, rpc_sender, mut rpc_receiver) = RpcClient::new(args.server).await;
    tokio::spawn(async move {
        if let Err(e) = rpc_client.run().await {
            error!("Rpc client error: {}", e);
        }
    });

    let (command_runner, command_sender, mut command_output_receiver) = CommandRunner::new().await;
    tokio::spawn(async move {
        if let Err(e) = command_runner.run().await {
            error!("Command runner error: {}", e);
        }
    });

    loop {
        tokio::select! {
            Some(event) = dbus_receiver.recv() => {
                let result = match event {
                    messages::DbusEvent::LoggedIn => rpc_sender.send(RpcCommand::LoggedIn).await,
                    messages::DbusEvent::LoggedOut => rpc_sender.send(RpcCommand::LoggedOut).await,
                };
                if let Err(e) = result {
                    debug!("Failed to handle dbus event: {}", e);
                }

            }
            Some(event) = rpc_receiver.recv() => {
                let result: anyhow::Result<()> = match event {
                    messages::RpcEvent::SetWallpaper(source) => dbus_sender
                        .send(DbusCommand::SetWallpaper(source))
                        .await
                        .map_err(Into::into),
                    messages::RpcEvent::SetContestUrl(url) => dbus_sender
                        .send(DbusCommand::SetContestUrl(url))
                        .await
                        .map_err(Into::into),
                    messages::RpcEvent::Login => dbus_sender
                        .send(DbusCommand::Login)
                        .await
                        .map_err(Into::into),
                    messages::RpcEvent::Logout => command_sender
                        .send(CommandRunnerCommand::Run {
                            id: None,
                            command: "systemctl restart greetd".to_string(),
                        })
                        .await
                        .map_err(Into::into),
                    messages::RpcEvent::LoginWithCredentials(username, password) => dbus_sender
                        .send(DbusCommand::LoginWithCredentials(username, password))
                        .await
                        .map_err(Into::into),
                    messages::RpcEvent::CustomCommand(id, command) => command_sender
                        .send(CommandRunnerCommand::Run {
                            id: Some(id),
                            command,
                        })
                        .await
                        .map_err(Into::into),
                    messages::RpcEvent::RequestLoginStatus => dbus_sender
                        .send(DbusCommand::GetLoginStatus)
                        .await
                        .map_err(Into::into),
                };

                if let Err(e) = result {
                    debug!("Failed to handle rpc event: {}", e);
                }
            }
            Some(event) = command_output_receiver.recv() => {
                let messages::CommandRunnerEvent::Result { id, output } = event;
                if let Some(id) = id && let Err(e) = rpc_sender.send(RpcCommand::CustomCommandOutput(id, output)).await {
                    debug!("Failed to send custom command output: {}", e);
                }

            }
        }
    }
}

fn setup_logging() {
    let filter = tracing_subscriber::filter::Targets::new()
        .with_target("loom_station", Level::TRACE)
        .with_default(Level::WARN);
    let registry = tracing_subscriber::registry().with(filter);
    if std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        registry.with(tracing_subscriber::fmt::layer()).init();
    } else {
        registry
            .with(tracing_journald::layer().expect("Failed to connect to journald"))
            .init();
    }
}
