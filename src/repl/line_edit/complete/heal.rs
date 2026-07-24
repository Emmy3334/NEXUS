//! Tab completion for `heal` / `doctor`.

const SUBCOMMANDS: &[&str] = &["help", "status"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Complete {
    Subcommand,
}

/// Classify completion when the line starts with `heal` or `doctor`.
pub(super) fn classify(words: &[&str]) -> Option<Complete> {
    match words.first().copied() {
        Some("heal" | "doctor") if words.len() == 1 => Some(Complete::Subcommand),
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
    }
}
