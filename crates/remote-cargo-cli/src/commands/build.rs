use remote_cargo_core::artifact_fetch::download_executables;

use super::remote_pipeline::sync_and_build;
use crate::utils::stage_timer::timed_stage;

pub fn execute(arguments: &[String]) -> anyhow::Result<()> {
    let pipeline = sync_and_build("build", arguments, arguments)?;
    let local_paths = timed_stage("download", || {
        download_executables(
            &pipeline.session,
            &pipeline.build.metadata.target_directory,
            &pipeline.project.target_directory,
            &pipeline.build.executables,
        )
    })?;
    for local_path in local_paths {
        eprintln!("cargo-remote: fetched {local_path}");
    }
    Ok(())
}
