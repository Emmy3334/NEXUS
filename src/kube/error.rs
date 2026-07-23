//! Map Kubernetes / connect failures to clearer shell messages.

use std::io;

/// Prefer a short actionable message for common offline-cluster errors.
pub(crate) fn format_err(err: &io::Error) -> String {
    let text = err.to_string();
    if looks_like_connect(&text) {
        return "cannot reach cluster (is minikube/Docker Desktop Kubernetes running? \
try: minikube start)"
            .into();
    }
    text
}

fn looks_like_connect(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("connect")
        || lower.contains("connection refused")
        || lower.contains("client error")
}
