//! Default / PATH / file match collection (zsh-like first-token rules).

use super::super::matchers::matches_prefix;
use super::super::paths::{collect_dir_matches, collect_file_matches, collect_path_commands};
use crate::builtins::NAMES;

/// Fallback when a specialized completer found nothing (argument position → files).
pub(super) fn fallback(prefix: &str, out: &mut Vec<String>) {
    if out.is_empty() && !prefix.starts_with('-') {
        files(prefix, out);
    }
}

/// Default matches: first token → commands; otherwise → files (unless path-like).
pub(super) fn matches(
    prefix: &str,
    path: &str,
    cmd_names: &[String],
    first_token: bool,
    out: &mut Vec<String>,
) {
    if prefix.contains('/') || prefix.starts_with('.') {
        collect_file_matches(prefix, out);
        return;
    }
    if first_token {
        commands(prefix, path, cmd_names, out);
    } else {
        files(prefix, out);
    }
}

fn commands(prefix: &str, path: &str, cmd_names: &[String], out: &mut Vec<String>) {
    for name in NAMES {
        if matches_prefix(name, prefix) {
            out.push((*name).to_owned());
        }
    }
    for name in cmd_names {
        if matches_prefix(name, prefix) {
            out.push(name.clone());
        }
    }
    collect_path_commands(prefix, path, out);
}

fn files(prefix: &str, out: &mut Vec<String>) {
    collect_file_matches(prefix, out);
}

/// `cd`/`pushd`/`rmdir` argument: directories only, like zsh.
pub(super) fn dirs(prefix: &str, out: &mut Vec<String>) {
    collect_dir_matches(prefix, out);
}
