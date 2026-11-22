use crate::RemoteCargoError;
use crate::local_project::LocalProject;
use crate::ssh_session::{SshSession, run_to_completion};
use crate::utils::shell_quote::shell_quote;

const TOOLCHAIN_FILES: [&str; 4] = [
    "rust-toolchain.toml",
    "rust-toolchain",
    ".cargo/config.toml",
    ".cargo/config",
];

pub fn upload_sources(
    session: &SshSession,
    project: &LocalProject,
) -> Result<(), RemoteCargoError> {
    let remote_source_root = project.remote_source_root();
    let mut rsync = session.rsync_command();
    rsync
        .arg(format!(
            "--rsync-path=mkdir -p {} && rsync",
            shell_quote(&remote_source_root)
        ))
        .args([
            "--relative",
            "--delete",
            "--exclude=.git",
            "--filter=:- .gitignore",
        ]);
    if let Ok(relative_target_directory) =
        project.target_directory.strip_prefix(&project.source_root)
    {
        rsync.arg(format!("--exclude=/{relative_target_directory}/"));
    }
    run_to_completion(
        rsync
            .args(local_upload_sources(project))
            .arg(session.remote_path(&format!("{remote_source_root}/"))),
        "rsync upload",
    )
}

fn local_upload_sources(project: &LocalProject) -> Vec<String> {
    let source_root = &project.source_root;
    let uploads_whole_source_root = project
        .relative_source_directories
        .iter()
        .any(|directory| directory.as_str().is_empty());
    if uploads_whole_source_root {
        return vec![format!("{source_root}/./")];
    }
    let toolchain_files = TOOLCHAIN_FILES
        .into_iter()
        .filter(|file| source_root.join(file).is_file())
        .map(|file| format!("{source_root}/./{file}"));
    project
        .relative_source_directories
        .iter()
        .map(|directory| format!("{source_root}/./{directory}/"))
        .chain(toolchain_files)
        .collect()
}
