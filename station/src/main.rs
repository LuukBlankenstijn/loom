use anyhow::Result;
use clap::Parser;
use tokio::sync::broadcast;
use tracing::Level;
use tracing::error;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::command::CommandRunner;
use crate::dbus::DbusClient;
use crate::messages::Message;
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

    /// Secret to use to authenticate with the backend
    #[arg(short, long)]
    auth: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_logging();
    let args = Args::parse();

    let (tx, _) = broadcast::channel::<Message>(32);

    let dbus_client = DbusClient::new(tx.clone()).await?;
    tokio::spawn(async move {
        if let Err(e) = dbus_client.run().await {
            error!("Dbus client error: {}", e);
        }
    });

    let rpc_client = RpcClient::new(args.server, args.auth, tx.clone());
    tokio::spawn(async move {
        if let Err(e) = rpc_client.run().await {
            error!("Rpc client error: {}", e);
        }
    });

    let command_runner = CommandRunner::new(tx);
    tokio::spawn(async move {
        if let Err(e) = command_runner.run().await {
            error!("Command runner error: {}", e);
        }
    });

    std::future::pending::<()>().await;
    Ok(())
}

fn setup_logging() {
    let filter = tracing_subscriber::filter::Targets::new()
        .with_target("loomd", Level::TRACE)
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
