use std::time::Duration;

use anyhow::Result;
use tokio::{
    process::Command,
    sync::mpsc::{Receiver, Sender, channel},
    time::timeout,
};
use tracing::debug;

use crate::messages::{CommandRunnerCommand, CommandRunnerEvent};

pub struct CommandRunner {
    receiver: Receiver<CommandRunnerCommand>,
    sender: Sender<CommandRunnerEvent>,
}

impl CommandRunner {
    pub async fn new() -> (
        Self,
        Sender<CommandRunnerCommand>,
        Receiver<CommandRunnerEvent>,
    ) {
        let (cmd_tx, cmd_rx) = channel(32);
        let (out_tx, out_rx) = channel(32);

        let runner = Self {
            receiver: cmd_rx,
            sender: out_tx,
        };

        (runner, cmd_tx, out_rx)
    }

    pub async fn run(mut self) -> Result<()> {
        while let Some(cmd) = self.receiver.recv().await {
            let CommandRunnerCommand::Run { id, command } = cmd;
            let tx = self.sender.clone();

            let future = execute_system_command(command);
            let msg = match timeout(Duration::from_secs(10), future).await {
                Ok(output) => CommandRunnerEvent::Result { id, output },
                Err(_) => CommandRunnerEvent::Result {
                    id,
                    output: "Command timed out".to_string(),
                },
            };
            if let Err(e) = tx.send(msg).await {
                debug!("failed to send command output: {}", e);
            }
        }
        Ok(())
    }
}

async fn execute_system_command(command_str: String) -> String {
    let result = Command::new("sh")
        .arg("-c")
        .arg(&command_str)
        .output()
        .await;

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

            if output.status.success() {
                // If stdout is empty, but command succeeded, show that
                if stdout.is_empty() {
                    "Command executed successfully (no output)".to_string()
                } else {
                    stdout
                }
            } else {
                format!(
                    "Command failed ({}).\nError: {}",
                    output.status,
                    if !stderr.is_empty() { stderr } else { stdout }
                )
            }
        }
        Err(e) => format!("Failed to invoke shell: {}", e),
    }
}
