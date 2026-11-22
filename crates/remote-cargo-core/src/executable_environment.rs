use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::{Artifact, Metadata};

pub fn cargo_run_environment(
    artifact: &Artifact,
    remote_metadata: &Metadata,
    local_workspace_root: &Utf8Path,
) -> Vec<(&'static str, String)> {
    let local_manifest_path = artifact
        .manifest_path
        .strip_prefix(&remote_metadata.workspace_root)
        .map_or_else(
            |_| artifact.manifest_path.clone(),
            |relative| local_workspace_root.join(relative),
        );
    let mut environment = vec![
        (
            "CARGO_MANIFEST_DIR",
            local_manifest_path
                .parent()
                .map(Utf8PathBuf::from)
                .unwrap_or_default()
                .into_string(),
        ),
        ("CARGO_MANIFEST_PATH", local_manifest_path.into_string()),
    ];
    if let Some(package) = remote_metadata
        .packages
        .iter()
        .find(|package| package.id == artifact.package_id)
    {
        let version = &package.version;
        environment.extend([
            ("CARGO_PKG_NAME", package.name.to_string()),
            ("CARGO_PKG_VERSION", version.to_string()),
            ("CARGO_PKG_VERSION_MAJOR", version.major.to_string()),
            ("CARGO_PKG_VERSION_MINOR", version.minor.to_string()),
            ("CARGO_PKG_VERSION_PATCH", version.patch.to_string()),
            ("CARGO_PKG_VERSION_PRE", version.pre.to_string()),
        ]);
    }
    environment
}
