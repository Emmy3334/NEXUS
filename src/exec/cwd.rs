//! Save / restore process working directory (subshell & command substitution).

use std::env;
use std::io::Write;
use std::path::PathBuf;

/// Snapshot the current directory if it can be read.
#[must_use]
pub(crate) fn save() -> Option<PathBuf> {
    env::current_dir().ok()
}

/// Restore `saved` cwd; log failures to `stderr` without aborting.
pub(crate) fn restore(saved: Option<PathBuf>, stderr: &mut impl Write) {
    if let Some(path) = saved {
        if let Err(err) = env::set_current_dir(&path) {
            let _ = writeln!(stderr, "nexus: failed to restore cwd: {err}");
        }
    }
}
