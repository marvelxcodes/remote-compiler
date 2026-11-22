use std::os::unix::process::CommandExt;
use std::process::Command;

use anyhow::{Context, bail};
use remote_cargo_core::artifact_fetch::download_executables;
use remote_cargo_core::executable_environment::cargo_run_environment;
use remote_cargo_core::executable_selection::select_executable;

use super::remote_pipeline::sync_and_build;
use crate::cargo_arguments::{executable_request, split_program_arguments};
use crate::utils::stage_timer::timed_stage;

pub fn execute(arguments: &[String]) -> anyhow::Result<()> {
    let (cargo_arguments, program_arguments) = split_program_arguments(arguments);
    let pipeline = sync_and_build("run", arguments, cargo_arguments)?;
    let selected_executable = select_executable(
        &pipeline.build.executables,
        &pipeline.build.metadata,
        executable_request(cargo_arguments),
    )?;
    let local_paths = timed_stage("download", || {
        download_executables(
            &pipeline.session,
            &pipeline.build.metadata.target_directory,
            &pipeline.project.target_directory,
            [selected_executable],
        )
    })?;
    let Some(local_executable) = local_paths.first() else {
        bail!("the selected target has no executable file");
    };
    let environment = cargo_run_environment(
        selected_executable,
        &pipeline.build.metadata,
        &pipeline.project.workspace_root,
    );
    Err(Command::new(local_executable)
        .args(program_arguments)
        .envs(environment)
        .exec())
    .with_context(|| format!("failed to execute {local_executable}"))
}
