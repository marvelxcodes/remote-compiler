use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::{Metadata, MetadataCommand};

use crate::RemoteCargoError;
use crate::remote_cache::REMOTE_WORKSPACES_DIRECTORY;
use crate::utils::common_ancestor::common_ancestor;
use crate::utils::containing_workspace_root::containing_workspace_root;
use crate::utils::remote_directory_name::remote_directory_name;

pub struct LocalProject {
    pub workspace_root: Utf8PathBuf,
    pub target_directory: Utf8PathBuf,
    pub source_root: Utf8PathBuf,
    pub relative_source_directories: Vec<Utf8PathBuf>,
    pub relative_working_directory: Utf8PathBuf,
}

impl LocalProject {
    pub fn discover() -> Result<Self, RemoteCargoError> {
        let metadata = MetadataCommand::new().exec()?;
        let working_directory = std::env::current_dir()?;
        let working_directory = Utf8Path::from_path(&working_directory)
            .filter(|directory| directory.starts_with(&metadata.workspace_root))
            .ok_or_else(|| {
                RemoteCargoError::WorkingDirectoryOutsideWorkspace(
                    working_directory.display().to_string(),
                )
            })?;

        let source_directories = local_source_directories(&metadata);
        let source_root = common_ancestor(source_directories.iter().copied())
            .filter(|ancestor| ancestor.parent().is_some())
            .ok_or_else(|| RemoteCargoError::NoCommonSourceDirectory {
                workspace_root: metadata.workspace_root.clone(),
            })?
            .to_owned();
        let relative_to_source_root = |path: &Utf8Path| {
            path.strip_prefix(&source_root)
                .map(Utf8Path::to_owned)
                .map_err(std::io::Error::other)
        };
        let relative_source_directories = source_directories
            .iter()
            .map(|directory| relative_to_source_root(directory))
            .collect::<Result<_, _>>()?;
        let relative_working_directory = relative_to_source_root(working_directory)?;

        Ok(Self {
            relative_source_directories,
            relative_working_directory,
            source_root,
            workspace_root: metadata.workspace_root,
            target_directory: metadata.target_directory,
        })
    }

    pub fn remote_source_root(&self) -> String {
        format!(
            "{REMOTE_WORKSPACES_DIRECTORY}/{}",
            remote_directory_name(&self.source_root)
        )
    }

    pub fn remote_working_directory(&self) -> String {
        let remote_source_root = self.remote_source_root();
        if self.relative_working_directory.as_str().is_empty() {
            remote_source_root
        } else {
            format!("{remote_source_root}/{}", self.relative_working_directory)
        }
    }
}

fn local_source_directories(metadata: &Metadata) -> Vec<&Utf8Path> {
    let mut source_directories: Vec<&Utf8Path> = metadata
        .packages
        .iter()
        .filter(|package| package.source.is_none())
        .filter_map(|package| package.manifest_path.parent())
        .filter(|package_directory| !package_directory.starts_with(&metadata.workspace_root))
        .map(containing_workspace_root)
        .chain([metadata.workspace_root.as_path()])
        .collect();
    source_directories.sort();
    source_directories.dedup_by(|later, earlier| later.starts_with(*earlier));
    source_directories
}
