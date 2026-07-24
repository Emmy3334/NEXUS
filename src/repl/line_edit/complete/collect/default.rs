//! Default / PATH / file match collection.

use super::super::matchers::matches_prefix;
use super::super::paths::{collect_file_matches, collect_path_commands};
use crate::builtins::NAMES;

pub(super) fn fallback(prefix: &str, path: &str, out: &mut Vec<String>) {
    if out.is_empty() && !prefix.starts_with('-') {
        matches(prefix, path, out);
    }
}

pub(super) fn matches(prefix: &str, path: &str, out: &mut Vec<String>) {
    if prefix.contains('/') || prefix.starts_with('.') {
        collect_file_matches(prefix, out);
        return;
    }
    for name in NAMES {
        if matches_prefix(name, prefix) {
            out.push((*name).to_owned());
        }
    }
    collect_path_commands(prefix, path, out);
    collect_file_matches(prefix, out);
}
