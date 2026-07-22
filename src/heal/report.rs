//! Classic spawn-failure messages (no heal backends involved).

use std::io::{self, Write};

/// Map a spawn `io::Error` to a shell status and stderr line.
pub(crate) fn report_spawn_failure(
    program: &str,
    err: &io::Error,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match err.kind() {
        io::ErrorKind::NotFound => {
            writeln!(stderr, "{program}: Command not found.")?;
            Ok(127)
        }
        io::ErrorKind::PermissionDenied => {
            writeln!(stderr, "{program}: Permission denied.")?;
            Ok(126)
        }
        _ => {
            writeln!(stderr, "{program}: {err}")?;
            Ok(1)
        }
    }
}
