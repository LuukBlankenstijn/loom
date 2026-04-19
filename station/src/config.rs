use std::process::Command;

use anyhow::Context;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Server to connect to
    #[arg(short, long)]
    pub server: String,

    /// Secret to use to authenticate with the backend
    #[arg(short, long)]
    pub auth: Option<String>,

    /// Command to run to get the auth_command
    /// If set the output of the command overwrites the auth argument
    #[arg(long)]
    pub auth_command: Option<String>,
}

impl Args {
    /// Parse and resolve the auth token
    pub fn parse_and_resolve() -> anyhow::Result<Self> {
        let mut args = Self::parse();
        args.resolve_auth().context("failed to resolve auth")?;
        Ok(args)
    }
    /// Returns the resolved auth token, running auth_command if set.
    fn resolve_auth(&mut self) -> anyhow::Result<()> {
        if let Some(cmd) = &self.auth_command {
            let output = Command::new("sh").arg("-c").arg(cmd).output()?;
            if !output.status.success() {
                return Err(anyhow::anyhow!(format!(
                    "auth_command failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
            let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
            self.auth = Some(token);
            Ok(())
        } else {
            Ok(())
        }
    }
}
