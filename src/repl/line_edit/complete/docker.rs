//! `@docker` Tab completion (subcommands + running container names).

use crate::heal;

const SUBCOMMANDS: &[&str] = &["ps", "logs", "help"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Complete {
    Subcommand,
    Container,
}

/// Classify `@docker` completion when the line starts with `@docker`.
pub(super) fn classify(words: &[&str]) -> Option<Complete> {
    if words.first().copied() != Some("@docker") {
        return None;
    }
    match words.get(1).copied() {
        None => Some(Complete::Subcommand),
        Some("logs") => Some(Complete::Container),
        _ => None,
    }
}

pub(super) fn collect(kind: &Complete, prefix: &str, out: &mut Vec<String>) {
    match kind {
        Complete::Subcommand => {
            for name in SUBCOMMANDS {
                if name.starts_with(prefix) {
                    out.push((*name).to_owned());
                }
            }
        }
        Complete::Container => out.extend(heal::running_names(prefix)),
    }
}
