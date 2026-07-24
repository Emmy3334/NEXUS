//! `@docker` Tab completion (subcommands, flags, running container names).

mod flags;

use super::matchers::matches_prefix;
use crate::heal;

const SUBCOMMANDS: &[&str] = &["ps", "logs", "help"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Complete {
    Subcommand,
    Ps,
    Logs,
    Help,
}

/// Classify `@docker` completion when the line starts with `@docker`.
pub(super) fn classify(words: &[&str]) -> Option<Complete> {
    if words.first().copied() != Some("@docker") {
        return None;
    }
    match words.get(1).copied() {
        None => Some(Complete::Subcommand),
        Some("ps") => Some(Complete::Ps),
        Some("logs") => Some(Complete::Logs),
        Some("help") => Some(Complete::Help),
        _ => None,
    }
}

pub(super) fn collect(kind: &Complete, prefix: &str, out: &mut Vec<String>) {
    if prefix.starts_with('-') {
        if let Some(verb) = verb_of(kind) {
            flags::collect(verb, prefix, out);
        }
        return;
    }
    match kind {
        Complete::Subcommand => push_static(SUBCOMMANDS, prefix, out),
        Complete::Ps | Complete::Help => {}
        Complete::Logs => out.extend(heal::running_names(prefix)),
    }
}

fn verb_of(kind: &Complete) -> Option<&'static str> {
    match kind {
        Complete::Ps => Some("ps"),
        Complete::Logs => Some("logs"),
        Complete::Subcommand | Complete::Help => None,
    }
}

fn push_static(items: &[&str], prefix: &str, out: &mut Vec<String>) {
    for item in items {
        if matches_prefix(item, prefix) {
            out.push((*item).to_owned());
        }
    }
}
