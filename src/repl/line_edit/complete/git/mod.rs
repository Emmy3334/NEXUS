//! Git-aware Tab completion (verbs, flags, local branches).

mod branches;
mod flags;

const VERBS: &[&str] = &[
    "commit",
    "push",
    "pull",
    "fetch",
    "log",
    "diff",
    "status",
    "stash",
    "checkout",
    "switch",
    "branch",
    "merge",
    "rebase",
    "cherry-pick",
    "reset",
    "restore",
    "revert",
];

/// Curated verbs that take branch names as a non-flag argument.
#[must_use]
pub(super) fn is_branch_verb(verb: &str) -> bool {
    matches!(
        verb,
        "checkout"
            | "switch"
            | "branch"
            | "merge"
            | "rebase"
            | "cherry-pick"
            | "reset"
            | "restore"
            | "revert"
    )
}

/// First non-flag word after `git`, if it is a curated verb.
#[must_use]
pub(super) fn verb_in(words: &[&str]) -> Option<&'static str> {
    if words.first().copied() != Some("git") {
        return None;
    }
    let verb = words.iter().skip(1).find(|w| !w.starts_with('-'))?;
    VERBS.iter().copied().find(|v| *v == *verb)
}

pub(super) fn collect_for_verb(verb: &str, prefix: &str, out: &mut Vec<String>) {
    if prefix.starts_with('-') {
        flags::collect(verb, prefix, out);
        return;
    }
    if is_branch_verb(verb) {
        branches::collect(prefix, out);
    }
}
