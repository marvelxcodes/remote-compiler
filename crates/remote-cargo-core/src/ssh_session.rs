use std::process::{Command, Stdio};

use crate::RemoteCargoError;
use crate::utils::shell_quote::shell_join;

pub const REMOTE_HOST_VARIABLE: &str = "REMOTE_CARGO_HOST";

const CONNECTION_SHARING_OPTIONS: [&str; 10] = [
    "-o",
    "ControlMaster=auto",
    "-o",
    "ControlPath=~/.ssh/remote-cargo-%C",
    "-o",
    "ControlPersist=10m",
    "-o",
    "ServerAliveInterval=2",
    "-o",
    "ServerAliveCountMax=2",
];

pub struct SshSession {
    destination: String,
}

impl SshSession {
    pub fn from_environment() -> Result<Self, RemoteCargoError> {
        std::env::var(REMOTE_HOST_VARIABLE)
            .ok()
            .filter(|destination| !destination.is_empty())
            .map(|destination| Self { destination })
            .ok_or(RemoteCargoError::MissingEnvironmentVariable(
                REMOTE_HOST_VARIABLE,
            ))
    }

    pub fn destination(&self) -> &str {
        &self.destination
    }

    pub fn is_reachable(&self) -> bool {
        Command::new("ssh")
            .args(CONNECTION_SHARING_OPTIONS)
            .args(["-o", "BatchMode=yes", "-o", "ConnectTimeout=3"])
            .arg(&self.destination)
            .arg("true")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }

    pub fn remote_command(&self, remote_script: &str) -> Command {
        let mut command = Command::new("ssh");
        command
            .args(CONNECTION_SHARING_OPTIONS)
            .arg(&self.destination)
            .arg(remote_script);
        command
    }

    pub fn interactive_remote_command(&self, remote_script: &str) -> Command {
        let mut command = Command::new("ssh");
        command
            .arg("-t")
            .args(CONNECTION_SHARING_OPTIONS)
            .arg(&self.destination)
            .arg(remote_script);
        command
    }

    pub fn rsync_command(&self) -> Command {
        let mut command = Command::new("rsync");
        command
            .arg("--rsh")
            .arg(format!("ssh {}", shell_join(CONNECTION_SHARING_OPTIONS)))
            .args(["--archive", "--compress"]);
        command
    }

    pub fn remote_path(&self, path: &str) -> String {
        format!("{}:{path}", self.destination)
    }
}

pub(crate) fn run_to_completion(
    command: &mut Command,
    program: &'static str,
) -> Result<(), RemoteCargoError> {
    let status = command
        .status()
        .map_err(|source| RemoteCargoError::ProcessStart { program, source })?;
    if status.success() {
        Ok(())
    } else {
        Err(RemoteCargoError::ProcessFailed { program, status })
    }
}
