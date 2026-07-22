//! Classify Docker heal errors for fallthrough vs hard failure.

use std::io;

/// True when the daemon could not exec argv (missing inside the image).
pub(super) fn is_missing_in_image(err: &io::Error) -> bool {
    let msg = err.to_string().to_ascii_lowercase();
    msg.contains("executable file not found") || msg.contains("not found in $path")
}
