//! Fetch or follow container logs by name or id.

use super::client;
use super::run::copy_logs;
use super::runtime::block_on;

use std::io::{self, Write};

/// Stream container logs into the shell writers (daemon/API errors propagate).
///
/// When `follow` is true, the stream continues until the container stops or the
/// API closes it (same idea as `docker logs -f`).
pub fn write_container_logs(
    target: &str,
    follow: bool,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<()> {
    let docker = client::connect()?;
    match block_on(copy_logs(&docker, target, follow, stdout, stderr)) {
        Ok(inner) => inner,
        Err(err) => Err(err),
    }
}
