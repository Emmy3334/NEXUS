//! Classify Kubernetes heal errors for fallthrough vs hard failure.

use std::io;

/// True when the container could not exec argv (missing inside the image).
pub(super) fn is_missing_in_image(err: &io::Error) -> bool {
    let msg = err.to_string().to_ascii_lowercase();
    msg.contains("executable file not found")
        || msg.contains("not found in $path")
        || msg.contains("no such file or directory")
}
