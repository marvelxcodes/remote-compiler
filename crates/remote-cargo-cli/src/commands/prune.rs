use remote_cargo_core::remote_cache::{remove_all_projects, remove_project};

use crate::server_connection::connect;

pub fn execute(arguments: &[String]) -> anyhow::Result<()> {
    let session = connect()?;
    match arguments {
        [] => {
            remove_all_projects(&session)?;
            eprintln!(
                "cargo-remote: removed all projects from {}",
                session.destination()
            );
        }
        [project_name] => {
            let removed_directory = remove_project(&session, project_name)?;
            eprintln!("cargo-remote: removed {removed_directory}");
        }
        _ => anyhow::bail!("usage: cargo prune [project-name]"),
    }
    Ok(())
}
