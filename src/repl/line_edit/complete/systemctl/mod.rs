//! Host `systemctl` verb flags.

mod flags;

const VERBS: &[&str] = &[
    "start",
    "stop",
    "restart",
    "status",
    "enable",
    "disable",
    "reload",
    "list-units",
    "list-unit-files",
    "daemon-reload",
    "show",
    "cat",
    "kill",
    "mask",
    "unmask",
    "is-active",
    "is-enabled",
    "is-failed",
];

/// Curated verb after `systemctl`, skipping leading flags (`--user`, …).
#[must_use]
pub(super) fn verb_in(words: &[&str]) -> Option<&'static str> {
    if words.first().copied() != Some("systemctl") {
        return None;
    }
    for w in words.iter().skip(1) {
        if w.starts_with('-') {
            continue;
        }
        if let Some(v) = VERBS.iter().copied().find(|v| *v == *w) {
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
