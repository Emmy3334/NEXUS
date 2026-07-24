//! Host `helm` verb flags.

mod flags;

const VERBS: &[&str] = &[
    "install",
    "upgrade",
    "uninstall",
    "list",
    "status",
    "rollback",
    "template",
    "lint",
    "repo",
    "search",
    "get",
    "history",
    "pull",
    "push",
    "show",
    "test",
    "dependency",
];

/// Curated verb after `helm`, skipping leading flags.
#[must_use]
pub(super) fn verb_in(words: &[&str]) -> Option<&'static str> {
    if words.first().copied() != Some("helm") {
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
