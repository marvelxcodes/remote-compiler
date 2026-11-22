use remote_cargo_core::remote_cache::clear_build_caches;

use crate::server_connection::connect;

pub fn execute() -> anyhow::Result<()> {
    clear_build_caches(&connect()?)?;
    Ok(())
}
