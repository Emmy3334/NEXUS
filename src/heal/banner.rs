//! Success banners for healed commands.

use std::io::{self, Write};

/// Print `nexus: healed via …` unless `quiet`.
pub(super) fn success(
    stderr: &mut dyn Write,
    quiet: bool,
    backend: &str,
    detail: Option<&str>,
) -> io::Result<()> {
    if quiet {
        return Ok(());
    }
    match detail {
        Some(d) => writeln!(stderr, "nexus: healed via {backend} ({d})"),
        None => writeln!(stderr, "nexus: healed via {backend}"),
    }
}
