//! Host `docker` verb flags (not the `@docker` builtin).

mod flags;

const VERBS: &[&str] = &["ps", "logs", "run", "exec", "rm", "images", "pull", "build"];

/// Curated verb after `docker`, skipping leading flags.
#[must_use]
pub(super) fn verb_in(words: &[&str]) -> Option<&'static str> {
    if words.first().copied() != Some("docker") {
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
