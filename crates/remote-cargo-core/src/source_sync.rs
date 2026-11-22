use crate::RemoteCargoError;
use crate::local_project::LocalProject;
use crate::ssh_session::{SshSession, run_to_completion};
use crate::utils::shell_quote::shell_quote;

pub fn upload_sources(
    session: &SshSession,
    project: &LocalProject,
) -> Result<(), RemoteCargoError> {
    let remote_workspace_root = project.remote_workspace_root();
    let mut rsync = session.rsync_command();
    rsync
        .arg(format!(
            "--rsync-path=mkdir -p {} && rsync",
            shell_quote(&remote_workspace_root)
        ))
        .args(["--delete", "--exclude=/.git/", "--filter=:- .gitignore"]);
    if let Ok(relative_target_directory) = project
        .target_directory
        .strip_prefix(&project.workspace_root)
    {
        rsync.arg(format!("--exclude=/{relative_target_directory}/"));
    }
    run_to_completion(
        rsync
            .arg(format!("{}/", project.workspace_root))
            .arg(session.remote_path(&format!("{remote_workspace_root}/"))),
        "rsync upload",
    )
}
