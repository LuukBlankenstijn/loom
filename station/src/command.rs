use std::time::Duration;

use anyhow::Result;
use tokio::{process::Command, sync::broadcast, time::timeout};

use crate::messages::Message;

pub struct CommandRunner {
    sender: broadcast::Sender<Message>,
    receiver: broadcast::Receiver<Message>,
}

impl CommandRunner {
    pub fn new(sender: broadcast::Sender<Message>) -> Self {
        let receiver = sender.subscribe();
        Self { sender, receiver }
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            let msg = match self.receiver.recv().await {
                Ok(msg) => msg,
                Err(broadcast::error::RecvError::Closed) => {
                    anyhow::bail!("broadcast channel closed")
                }
                Err(_) => continue,
            };

            match msg {
                Message::RunCommand {
                    id,
                    command,
                    admin_id,
                } => {
                    let tx = self.sender.clone();
                    tokio::spawn(async move {
                        let output = run_with_timeout(&command).await;
                        let _ = tx.send(Message::CommandOutput {
                            id,
                            output,
                            admin_id,
                        });
                    });
                }
                Message::Logout => {
                    tokio::spawn(async move {
                        run_with_timeout("systemctl restart greetd").await;
                    });
                }
                _ => {}
            }
        }
    }
}

async fn run_with_timeout(command: &str) -> String {
    match timeout(Duration::from_secs(10), execute_system_command(command)).await {
        Ok(output) => output,
        Err(_) => "Command timed out".to_string(),
    }
}

async fn execute_system_command(command_str: &str) -> String {
    let result = Command::new("/bin/sh")
        .arg("-c")
        .arg(command_str)
        .output()
        .await;

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

            if output.status.success() {
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
