//! Common flags for curated `git` verbs.

pub(super) fn flags_for(verb: &str) -> Option<&'static [&'static str]> {
    match verb {
        "commit" => Some(COMMIT),
        "push" => Some(PUSH),
        "pull" => Some(PULL),
        "fetch" => Some(FETCH),
        "log" => Some(LOG),
        "diff" => Some(DIFF),
        "status" => Some(STATUS),
        "stash" => Some(STASH),
        "checkout" => Some(CHECKOUT),
        "switch" => Some(SWITCH),
        "branch" => Some(BRANCH),
        "merge" => Some(MERGE),
        "rebase" => Some(REBASE),
        "cherry-pick" => Some(CHERRY_PICK),
        "reset" => Some(RESET),
        "restore" => Some(RESTORE),
        "revert" => Some(REVERT),
        _ => None,
    }
}

pub(super) fn collect(verb: &str, prefix: &str, out: &mut Vec<String>) {
    let Some(flags) = flags_for(verb) else {
        return;
    };
    for flag in flags {
        if flag.starts_with(prefix) {
            out.push((*flag).to_owned());
        }
    }
}

const COMMIT: &[&str] = &[
    "-a",
    "-m",
    "-p",
    "-v",
    "--amend",
    "--no-edit",
    "--allow-empty",
];
const PUSH: &[&str] = &[
    "-u",
    "-f",
    "--force",
    "--force-with-lease",
    "--tags",
    "--all",
    "--dry-run",
];
const PULL: &[&str] = &["--rebase", "--ff-only", "--no-rebase", "--all", "--tags"];
const FETCH: &[&str] = &["--all", "--prune", "--tags", "--dry-run"];
const LOG: &[&str] = &[
    "--oneline",
    "--graph",
    "--stat",
    "--all",
    "--decorate",
    "-p",
    "-n",
];
const DIFF: &[&str] = &[
    "--stat",
    "--cached",
    "--staged",
    "--name-only",
    "--name-status",
    "-p",
];
const STATUS: &[&str] = &["-s", "-b", "--short", "--porcelain", "--ignored"];
const STASH: &[&str] = &[
    "push",
    "pop",
    "list",
    "apply",
    "drop",
    "show",
    "-u",
    "--include-untracked",
];
const CHECKOUT: &[&str] = &["-b", "-B", "-f", "--orphan", "--detach", "-"];
const SWITCH: &[&str] = &["-c", "-C", "-f", "-d", "--detach", "--orphan"];
const BRANCH: &[&str] = &[
    "-a",
    "-d",
    "-D",
    "-m",
    "-M",
    "-v",
    "-r",
    "--list",
    "--show-current",
];
const MERGE: &[&str] = &[
    "--no-ff",
    "--ff-only",
    "--squash",
    "--abort",
    "--continue",
    "-m",
];
const REBASE: &[&str] = &[
    "-i",
    "--abort",
    "--continue",
    "--skip",
    "--onto",
    "--autosquash",
];
const CHERRY_PICK: &[&str] = &["-x", "-n", "--no-commit", "--abort", "--continue"];
const RESET: &[&str] = &["--hard", "--soft", "--mixed", "-q"];
const RESTORE: &[&str] = &["--staged", "--worktree", "-s", "--source"];
const REVERT: &[&str] = &["--no-commit", "--abort", "--continue", "-n"];
