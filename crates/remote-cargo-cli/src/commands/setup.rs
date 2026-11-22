use anyhow::Context;
use remote_cargo_core::ssh_session::SshSession;

const SERVER_SETUP_SCRIPT: &str = include_str!("setup_server.sh");

pub fn execute() -> anyhow::Result<()> {
    let session = SshSession::from_environment()?;
    let status = session
        .interactive_remote_command(SERVER_SETUP_SCRIPT)
        .status()
        .context("failed to start ssh")?;
    anyhow::ensure!(status.success(), "server setup failed with {status}");
    eprintln!("cargo-remote: {} is ready", session.destination());
    Ok(())
}
