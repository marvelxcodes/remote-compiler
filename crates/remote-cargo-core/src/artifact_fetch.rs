use cargo_metadata::Artifact;
use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};

use crate::RemoteCargoError;
use crate::ssh_session::{SshSession, run_to_completion};

pub fn download_executables<'a>(
    session: &SshSession,
    remote_target_directory: &Utf8Path,
    local_target_directory: &Utf8Path,
    executables: impl IntoIterator<Item = &'a Artifact>,
) -> Result<Vec<Utf8PathBuf>, RemoteCargoError> {
    let mut remote_sources = Vec::new();
    let mut local_paths = Vec::new();
    for remote_executable in executables
        .into_iter()
        .filter_map(|artifact| artifact.executable.as_ref())
    {
        let relative_path = remote_executable
            .strip_prefix(remote_target_directory)
            .map_err(|_| RemoteCargoError::ExecutableOutsideTargetDirectory {
                executable: remote_executable.clone(),
                target_directory: remote_target_directory.to_owned(),
            })?;
        remote_sources
            .push(session.remote_path(&format!("{remote_target_directory}/./{relative_path}")));
        local_paths.push(local_target_directory.join(relative_path));
    }

    if !remote_sources.is_empty() {
        run_to_completion(
            session
                .rsync_command()
                .args(["--relative", "--mkpath"])
                .args(&remote_sources)
                .arg(format!("{local_target_directory}/")),
            "rsync download",
        )?;
    }
    Ok(local_paths)
}
