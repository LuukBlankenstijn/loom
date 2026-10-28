use std::process::Command;

use anyhow::Context;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
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

    /// Path (or PATH-resolvable name) of the registration tool binary that
    /// loomd launches into the active graphical session on request.
    #[arg(long, default_value = "loom-station-registration")]
    pub registration_tool: String,

    /// Free-form label reported to the backend, e.g. "team" or "judgehost".
    /// Trimmed and lowercased; an empty value means no label.
    #[arg(long)]
    pub label: Option<String>,

    /// Disable the greeter integration: no dbus connection to the greeter,
    /// the station always reports itself as logged in and logout requests
    /// are ignored.
    #[arg(long)]
    pub no_greeter: bool,

    /// Disable launching the registration tool on request.
    #[arg(long)]
    pub no_registration_tool: bool,
}

impl Args {
    /// Parse the arguments, normalise the label and resolve the auth token
    pub fn parse_and_resolve() -> anyhow::Result<Self> {
        let mut args = Self::parse();
        args.label = args.label.as_deref().and_then(normalize_label);
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

fn normalize_label(label: &str) -> Option<String> {
    let label = label.trim().to_lowercase();
    (!label.is_empty()).then_some(label)
}
