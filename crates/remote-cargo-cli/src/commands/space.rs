use remote_cargo_core::remote_cache::print_space_usage;

use crate::server_connection::connect;

pub fn execute() -> anyhow::Result<()> {
    print_space_usage(&connect()?)?;
    Ok(())
}
