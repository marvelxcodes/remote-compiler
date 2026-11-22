use std::process::ExitStatus;

use cargo_metadata::camino::Utf8PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum RemoteCargoError {
    #[error("environment variable {0} is not set")]
    MissingEnvironmentVariable(&'static str),

    #[error("failed to start {program}: {source}")]
    ProcessStart {
        program: &'static str,
        source: std::io::Error,
    },

    #[error("{program} failed with {status}")]
    ProcessFailed {
        program: &'static str,
        status: ExitStatus,
    },

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Metadata(#[from] cargo_metadata::Error),

    #[error("current directory {0} is not valid UTF-8 or is outside the workspace")]
    WorkingDirectoryOutsideWorkspace(String),

    #[error(
        "path dependency `{dependency_name}` at {dependency_path} is outside the workspace root \
         {workspace_root} and is not synced to the server"
    )]
    PathDependencyOutsideWorkspace {
        dependency_name: String,
        dependency_path: Utf8PathBuf,
        workspace_root: Utf8PathBuf,
    },

    #[error("executable {executable} is outside the remote target directory {target_directory}")]
    ExecutableOutsideTargetDirectory {
        executable: Utf8PathBuf,
        target_directory: Utf8PathBuf,
    },

    #[error("the build produced no executable to run")]
    NoExecutable,

    #[error("no executable target named `{0}` was built")]
    ExecutableNotFound(String),

    #[error(
        "could not determine which executable to run, use --bin or set `default-run` \
         (available: {0})"
    )]
    AmbiguousExecutable(String),
}

impl RemoteCargoError {
    pub fn exit_code(&self) -> Option<i32> {
        match self {
            Self::ProcessFailed { status, .. } => status.code(),
            _ => None,
        }
    }
}
