use remote_cargo_core::local_project::LocalProject;
use remote_cargo_core::remote_build::{RemoteBuild, build_remotely};
use remote_cargo_core::source_sync::upload_sources;
use remote_cargo_core::ssh_session::SshSession;

use crate::server_connection::connect_or_run_locally;
use crate::utils::stage_timer::timed_stage;

pub struct RemotePipeline {
    pub session: SshSession,
    pub project: LocalProject,
    pub build: RemoteBuild,
}

pub fn sync_and_build(
    subcommand: &str,
    original_arguments: &[String],
    cargo_build_arguments: &[String],
) -> anyhow::Result<RemotePipeline> {
    let session = timed_stage("connect", || {
        connect_or_run_locally(subcommand, original_arguments)
    })?;
    let project = LocalProject::discover()?;
    timed_stage("upload", || upload_sources(&session, &project))?;
    let build = timed_stage("remote build", || {
        build_remotely(&session, &project, cargo_build_arguments)
    })?;
    Ok(RemotePipeline {
        session,
        project,
        build,
    })
}
