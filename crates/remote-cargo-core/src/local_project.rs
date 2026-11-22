use cargo_metadata::MetadataCommand;
use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};

use crate::RemoteCargoError;
use crate::utils::remote_directory_name::remote_directory_name;

const REMOTE_WORKSPACES_DIRECTORY: &str = ".cache/remote-cargo";

pub struct LocalProject {
    pub workspace_root: Utf8PathBuf,
    pub target_directory: Utf8PathBuf,
    pub relative_working_directory: Utf8PathBuf,
}

impl LocalProject {
    pub fn discover() -> Result<Self, RemoteCargoError> {
        let metadata = MetadataCommand::new().no_deps().exec()?;
        let working_directory = std::env::current_dir()?;
        let relative_working_directory = Utf8Path::from_path(&working_directory)
            .and_then(|directory| directory.strip_prefix(&metadata.workspace_root).ok())
            .ok_or_else(|| {
                RemoteCargoError::WorkingDirectoryOutsideWorkspace(
                    working_directory.display().to_string(),
                )
            })?
            .to_owned();

        for package in metadata.workspace_packages() {
            for dependency in &package.dependencies {
                if let Some(dependency_path) = &dependency.path
                    && !dependency_path.starts_with(&metadata.workspace_root)
                {
                    return Err(RemoteCargoError::PathDependencyOutsideWorkspace {
                        dependency_name: dependency.name.clone(),
                        dependency_path: dependency_path.clone(),
                        workspace_root: metadata.workspace_root.clone(),
                    });
                }
            }
        }

        Ok(Self {
            workspace_root: metadata.workspace_root,
            target_directory: metadata.target_directory,
            relative_working_directory,
        })
    }

    pub fn remote_workspace_root(&self) -> String {
        format!(
            "{REMOTE_WORKSPACES_DIRECTORY}/{}",
            remote_directory_name(&self.workspace_root)
        )
    }

    pub fn remote_working_directory(&self) -> String {
        let remote_workspace_root = self.remote_workspace_root();
        if self.relative_working_directory.as_str().is_empty() {
            remote_workspace_root
        } else {
            format!(
                "{remote_workspace_root}/{}",
                self.relative_working_directory
            )
        }
    }
}
