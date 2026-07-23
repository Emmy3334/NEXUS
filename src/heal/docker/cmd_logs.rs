//! Fetch logs for a running or stopped container by name or id.

use super::client;
use super::run::copy_logs;
use super::runtime::block_on;

use std::io::{self, Write};

/// Stream container logs into the shell writers (daemon/API errors propagate).
pub fn write_container_logs(
    target: &str,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<()> {
    let docker = client::connect()?;
    match block_on(copy_logs(&docker, target, stdout, stderr)) {
        Ok(inner) => inner,
        Err(err) => Err(err),
    }
}
