use std::convert::Infallible;
use std::os::unix::process::CommandExt;
use std::process::Command;

use anyhow::Context;
use remote_cargo_core::ssh_session::SshSession;

pub fn connect_or_run_locally(
    subcommand: &str,
    arguments: &[String],
) -> anyhow::Result<SshSession> {
    let reason = match SshSession::from_environment() {
        Ok(session) if session.is_reachable() => return Ok(session),
        Ok(session) => format!("cannot connect to {}", session.destination()),
        Err(error) => error.to_string(),
    };
    eprintln!("cargo-remote: {reason}, building locally");
    match replace_process_with_local_cargo(subcommand, arguments)? {}
}

fn replace_process_with_local_cargo(
    subcommand: &str,
    arguments: &[String],
) -> anyhow::Result<Infallible> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    Err(Command::new(cargo).arg(subcommand).args(arguments).exec())
        .context("failed to run local cargo")
}
