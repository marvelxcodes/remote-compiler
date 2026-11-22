use remote_cargo_core::local_project::LocalProject;
use remote_cargo_core::remote_cache::clean_project;

use crate::server_connection::{connect_or_run_locally, replace_process_with_local_cargo};

pub fn execute(arguments: &[String]) -> anyhow::Result<()> {
    let session = connect_or_run_locally("clean", arguments)?;
    clean_project(&session, &LocalProject::discover()?, arguments)?;
    match replace_process_with_local_cargo("clean", arguments)? {}
}
