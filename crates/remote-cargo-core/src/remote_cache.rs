use crate::RemoteCargoError;
use crate::local_project::LocalProject;
use crate::ssh_session::{SshSession, run_to_completion};
use crate::utils::remote_directory_name::remote_directory_name;
use crate::utils::shell_quote::{shell_join, shell_quote};

pub(crate) const REMOTE_WORKSPACES_DIRECTORY: &str = ".cache/remote-cargo";

const LOAD_CARGO_ENVIRONMENT: &str = r#"[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env""#;

pub fn clean_project(
    session: &SshSession,
    project: &LocalProject,
    cargo_clean_arguments: &[String],
) -> Result<(), RemoteCargoError> {
    let remote_working_directory = shell_quote(&project.remote_working_directory()).into_owned();
    let script = format!(
        "{LOAD_CARGO_ENVIRONMENT}; if [ -d {remote_working_directory} ]; then \
         cd {remote_working_directory} && cargo clean {}; \
         else echo 'project is not on the server' >&2; fi",
        shell_join(cargo_clean_arguments.iter().map(String::as_str)),
    );
    run_to_completion(&mut session.remote_command(&script), "remote cargo clean")
}

pub fn clear_build_caches(session: &SshSession) -> Result<(), RemoteCargoError> {
    let script = format!(
        r#"{LOAD_CARGO_ENVIRONMENT}; cd {REMOTE_WORKSPACES_DIRECTORY} 2>/dev/null || exit 0
        for project in */; do
            [ -d "$project" ] || continue
            echo "clearing ${{project%/}}" >&2
            (cd "$project" && cargo clean --quiet 2>/dev/null) || rm -rf "${{project}}target"
        done"#
    );
    run_to_completion(&mut session.remote_command(&script), "remote cache clear")
}

pub fn print_space_usage(session: &SshSession) -> Result<(), RemoteCargoError> {
    let script = format!(
        "cd {REMOTE_WORKSPACES_DIRECTORY} 2>/dev/null && [ -n \"$(ls -A)\" ] \
         || {{ echo 'no projects on the server'; exit 0; }}; du -sch -- * | sort -h"
    );
    run_to_completion(&mut session.remote_command(&script), "remote du")
}

pub fn remove_all_projects(session: &SshSession) -> Result<(), RemoteCargoError> {
    run_to_completion(
        &mut session.remote_command(&format!("rm -rf {REMOTE_WORKSPACES_DIRECTORY}")),
        "remote prune",
    )
}

pub fn remove_project(
    session: &SshSession,
    project_query: &str,
) -> Result<String, RemoteCargoError> {
    let listing = session
        .remote_command(&format!(
            "ls -1 {REMOTE_WORKSPACES_DIRECTORY} 2>/dev/null; true"
        ))
        .output()
        .map_err(|source| RemoteCargoError::ProcessStart {
            program: "ssh",
            source,
        })?;
    if !listing.status.success() {
        return Err(RemoteCargoError::ProcessFailed {
            program: "remote ls",
            status: listing.status,
        });
    }
    let listing = String::from_utf8_lossy(&listing.stdout);
    let project_directory = find_project_directory(listing.lines(), project_query)?.to_owned();
    run_to_completion(
        &mut session.remote_command(&format!(
            "rm -rf -- {}",
            shell_quote(&format!(
                "{REMOTE_WORKSPACES_DIRECTORY}/{project_directory}"
            ))
        )),
        "remote prune",
    )?;
    Ok(project_directory)
}

fn find_project_directory<'a>(
    project_directories: impl Iterator<Item = &'a str>,
    project_query: &str,
) -> Result<&'a str, RemoteCargoError> {
    let normalized_query = remote_directory_name(project_query.into());
    let suffix = format!("_{normalized_query}");
    let candidates: Vec<&str> = project_directories
        .filter(|directory| *directory == normalized_query || directory.ends_with(&suffix))
        .collect();
    if let Some(exact_match) = candidates
        .iter()
        .find(|directory| **directory == normalized_query)
    {
        return Ok(exact_match);
    }
    match candidates.as_slice() {
        [] => Err(RemoteCargoError::ProjectNotFound(project_query.to_owned())),
        [single_match] => Ok(single_match),
        _ => Err(RemoteCargoError::AmbiguousProject {
            project_query: project_query.to_owned(),
            matches: candidates.join(", "),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIRECTORIES: [&str; 3] = ["e_Projects_api", "home_me_work_api", "e_Projects_web-app"];

    #[test]
    fn matches_project_by_last_path_component() {
        let found = find_project_directory(DIRECTORIES.into_iter(), "web-app");
        assert!(matches!(found, Ok("e_Projects_web-app")));
    }

    #[test]
    fn prefers_exact_directory_name() {
        let found = find_project_directory(DIRECTORIES.into_iter(), "home_me_work_api");
        assert!(matches!(found, Ok("home_me_work_api")));
    }

    #[test]
    fn rejects_ambiguous_and_unknown_names() {
        let ambiguous = find_project_directory(DIRECTORIES.into_iter(), "api");
        let unknown = find_project_directory(DIRECTORIES.into_iter(), "missing");
        assert!(matches!(
            ambiguous,
            Err(RemoteCargoError::AmbiguousProject { .. })
        ));
        assert!(matches!(unknown, Err(RemoteCargoError::ProjectNotFound(_))));
    }
}
