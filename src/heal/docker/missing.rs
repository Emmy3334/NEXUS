//! Classify Docker heal errors for fallthrough vs hard failure.

use std::io;

/// True when the next heal backend (or classic not-found) should run.
pub(super) fn should_decline(err: &io::Error) -> bool {
    let msg = err.to_string().to_ascii_lowercase();
    msg.contains("executable file not found")
        || msg.contains("not found in $path")
        || msg.contains("no such file or directory")
        || msg.contains("invalid volume")
        || msg.contains("invalid mount")
}
