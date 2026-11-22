use std::io::{BufRead, BufReader, IsTerminal};
use std::process::Stdio;

use cargo_metadata::{Artifact, Message, Metadata, MetadataCommand};

use crate::RemoteCargoError;
use crate::local_project::LocalProject;
use crate::ssh_session::SshSession;
use crate::utils::shell_quote::{shell_join, shell_quote};

const DEFAULT_PROGRESS_WIDTH: &str = "100";

pub struct RemoteBuild {
    pub metadata: Metadata,
    pub executables: Vec<Artifact>,
}

pub fn build_remotely(
    session: &SshSession,
    project: &LocalProject,
    cargo_build_arguments: &[String],
) -> Result<RemoteBuild, RemoteCargoError> {
    let mut remote_process = session
        .remote_command(&remote_build_script(project, cargo_build_arguments))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|source| RemoteCargoError::ProcessStart {
            program: "ssh",
            source,
        })?;
    let mut remote_output = BufReader::new(
        remote_process
            .stdout
            .take()
            .ok_or_else(|| std::io::Error::other("ssh output is not captured"))?,
    );

    let mut metadata_line = String::new();
    remote_output.read_line(&mut metadata_line)?;
    let executables = Message::parse_stream(remote_output)
        .filter_map(|message| match message {
            Ok(Message::CompilerArtifact(artifact)) if artifact.executable.is_some() => {
                Some(Ok(artifact))
            }
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<_>, _>>()?;

    let status = remote_process.wait()?;
    if !status.success() {
        return Err(RemoteCargoError::ProcessFailed {
            program: "remote cargo build",
            status,
        });
    }

    Ok(RemoteBuild {
        metadata: MetadataCommand::parse(metadata_line)?,
        executables,
    })
}

fn remote_build_script(project: &LocalProject, cargo_build_arguments: &[String]) -> String {
    format!(
        "[ -f \"$HOME/.cargo/env\" ] && . \"$HOME/.cargo/env\"; cd {} && cargo metadata --no-deps --format-version 1 \
         && exec env {} cargo build {} --message-format=json-render-diagnostics",
        shell_quote(&project.remote_working_directory()),
        terminal_environment(),
        shell_join(cargo_build_arguments.iter().map(String::as_str)),
    )
}

fn terminal_environment() -> String {
    if !std::io::stderr().is_terminal() {
        return String::new();
    }
    let progress_width = std::env::var("COLUMNS").unwrap_or_else(|_| DEFAULT_PROGRESS_WIDTH.into());
    format!(
        "CARGO_TERM_COLOR=always CARGO_TERM_PROGRESS_WHEN=always CARGO_TERM_PROGRESS_WIDTH={}",
        shell_quote(&progress_width)
    )
}
