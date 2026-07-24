//! Host `kubectl` second-level completion (kinds + verb flags).

mod flags;
mod kinds;

const VERBS: &[&str] = &["get", "describe", "logs", "apply", "delete", "exec"];

/// Curated verb after `kubectl`, skipping global flags and their values.
#[must_use]
pub(super) fn verb_in(words: &[&str]) -> Option<&'static str> {
    if words.first().copied() != Some("kubectl") {
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

pub(super) fn collect_for_verb(verb: &str, words: &[&str], prefix: &str, out: &mut Vec<String>) {
    if prefix.starts_with('-') {
        flags::collect(verb, prefix, out);
        return;
    }
    if let Some(kind) = kinds::awaiting(words) {
        kinds::collect(&kind, prefix, out);
    }
}
