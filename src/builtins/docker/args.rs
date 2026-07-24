//! Shared flag / positional helpers for `@docker`.

/// Whether `-f` / `--follow` appears among args.
#[must_use]
pub(super) fn wants_follow(args: &[String]) -> bool {
    args.iter().any(|a| matches!(a.as_str(), "-f" | "--follow"))
}

/// Whether `-a` / `--all` appears among args.
#[must_use]
pub(super) fn wants_all(args: &[String]) -> bool {
    args.iter().any(|a| matches!(a.as_str(), "-a" | "--all"))
}

/// First non-flag positional after `skip` leading args (subcommand word).
#[must_use]
pub(super) fn first_positional(args: &[String], skip: usize) -> Option<&str> {
    let mut i = skip;
    while i < args.len() {
        match args[i].as_str() {
            "-f" | "--follow" | "-a" | "-q" | "--all" | "--quiet" | "-t" | "--timestamps"
            | "--details" => i += 1,
            "--tail" | "--filter" | "--format" => {
                i += 1;
                if args.get(i).is_some_and(|n| !n.starts_with('-')) {
                    i += 1;
                }
            }
            flag if flag.starts_with('-') => i += 1,
            other => return Some(other),
        }
    }
    None
}
