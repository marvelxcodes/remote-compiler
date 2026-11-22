use cargo_metadata::{Artifact, Metadata, TargetKind};

use crate::RemoteCargoError;

#[derive(Clone, Copy)]
pub enum ExecutableRequest<'a> {
    Default,
    Binary(&'a str),
    Example(&'a str),
}

pub fn select_executable<'a>(
    executables: &'a [Artifact],
    metadata: &Metadata,
    request: ExecutableRequest<'_>,
) -> Result<&'a Artifact, RemoteCargoError> {
    let find_named = |kind: TargetKind, name: &str| {
        executables
            .iter()
            .find(|artifact| artifact.target.name == name && artifact.target.kind.contains(&kind))
            .ok_or_else(|| RemoteCargoError::ExecutableNotFound(name.to_owned()))
    };

    match request {
        ExecutableRequest::Binary(name) => find_named(TargetKind::Bin, name),
        ExecutableRequest::Example(name) => find_named(TargetKind::Example, name),
        ExecutableRequest::Default => {
            let binaries: Vec<&Artifact> = executables
                .iter()
                .filter(|artifact| artifact.target.kind.contains(&TargetKind::Bin))
                .collect();
            match binaries.as_slice() {
                [] => Err(RemoteCargoError::NoExecutable),
                [only_binary] => Ok(only_binary),
                [first_binary, ..] => {
                    let all_from_one_package = binaries
                        .iter()
                        .all(|artifact| artifact.package_id == first_binary.package_id);
                    let default_run = metadata
                        .packages
                        .iter()
                        .filter(|_| all_from_one_package)
                        .find(|package| package.id == first_binary.package_id)
                        .and_then(|package| package.default_run.as_deref());
                    default_run
                        .and_then(|name| {
                            binaries
                                .iter()
                                .copied()
                                .find(|artifact| artifact.target.name == name)
                        })
                        .ok_or_else(|| {
                            RemoteCargoError::AmbiguousExecutable(
                                binaries
                                    .iter()
                                    .map(|artifact| artifact.target.name.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", "),
                            )
                        })
                }
            }
        }
    }
}
