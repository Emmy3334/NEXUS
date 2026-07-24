//! Host `gcloud` group flags.

mod flags;

const GROUPS: &[&str] = &[
    "compute",
    "container",
    "run",
    "iam",
    "auth",
    "config",
    "projects",
    "storage",
    "functions",
    "sql",
    "logging",
    "pubsub",
    "secrets",
    "services",
];

/// Curated group after `gcloud`, skipping leading global flags.
#[must_use]
pub(super) fn verb_in(words: &[&str]) -> Option<&'static str> {
    if words.first().copied() != Some("gcloud") {
        return None;
    }
    for w in words.iter().skip(1) {
        if w.starts_with('-') {
            continue;
        }
        if let Some(v) = GROUPS.iter().copied().find(|v| *v == *w) {
            return Some(v);
        }
    }
    None
}

pub(super) fn collect_for_verb(verb: &str, prefix: &str, out: &mut Vec<String>) {
    if prefix.starts_with('-') {
        flags::collect(verb, prefix, out);
    }
}
