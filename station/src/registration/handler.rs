use std::{process::Child, time::Duration};

use anyhow::Result;
use tokio::sync::broadcast;
use tracing::error;
use zbus::Connection;

use crate::messages::Message;

const TERMINATE_GRACE: Duration = Duration::from_secs(2);

pub struct RegistrationToolRunner {
    receiver: broadcast::Receiver<Message>,
    current: Option<Child>,
    args: crate::config::Args,
}

impl RegistrationToolRunner {
    pub fn new(sender: broadcast::Sender<Message>, args: crate::config::Args) -> Self {
        let receiver = sender.subscribe();
        Self {
            receiver,
            current: None,
            args,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        let mut reap = tokio::time::interval(Duration::from_secs(2));
        reap.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                msg = self.receiver.recv() => {
                    let msg = match msg {
                        Ok(msg) => msg,
                        Err(broadcast::error::RecvError::Closed) => {
                            anyhow::bail!("broadcast channel closed")
                        }
                        Err(_) => continue,
                    };

                    // A single command failing (e.g. no graphical session yet)
                    // must not tear down the runner — log and keep listening.
                    match msg {
                        Message::StartRegistrationTool => {
                            if let Err(e) = self.start().await {
                                error!("failed to start registration tool: {e:#}");
                            }
                        }
                        Message::StopRegistrationTool => {
                            if let Err(e) = self.stop().await {
                                error!("failed to stop registration tool: {e:#}");
                            }
                        }
                        _ => {}
                    }
                }
                _ = reap.tick() => {
                    // Reap a self-exited child so it doesn't linger as a zombie
                    // and a later Start doesn't signal a dead pid.
                    if let Some(child) = self.current.as_mut()
                        && matches!(child.try_wait(), Ok(Some(_)))
                    {
                        self.current = None;
                    }
                }
            }
        }
    }

    async fn start(&mut self) -> Result<()> {
        // Replace any previous instance so we never orphan a child.
        self.stop().await?;

        // Pass server/token via env, not argv: the child runs as the session
        // user and its /proc/<pid>/cmdline is world-readable, whereas environ is
        // owner-only.
        let mut env: Vec<(&str, &str)> = vec![("LOOM_SERVER", self.args.server.as_str())];
        if let Some(auth) = &self.args.auth {
            env.push(("LOOM_AUTH", auth.as_str()));
        }

        let conn = Connection::system().await?;
        let session = super::sys::find_target_session(&conn).await?;
        let child = super::sys::launch_gui(&session, &self.args.registration_tool, &env)?;
        self.current = Some(child);
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(mut current) = self.current.take() {
            super::sys::terminate_child(&mut current, TERMINATE_GRACE).await?;
        }
        Ok(())
    }
}
