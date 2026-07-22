//! Tab completion for the current token.

mod paths;

use self::paths::{collect_file_matches, collect_path_commands};

const BUILTINS: &[&str] = &[
    "alias", "bg", "cd", "env", "exit", "fg", "history", "jobs", "set", "setenv", "source",
    "unalias", "unset", "unsetenv",
];

/// Replace the token under the cursor; returns display lines for ambiguous matches.
pub(super) fn complete(buffer: &mut String, cursor: &mut usize) -> Vec<String> {
    let (start, prefix) = token_at(buffer, *cursor);
    let matches = collect_matches(&prefix);
    match matches.as_slice() {
        [] => Vec::new(),
        [only] => {
            apply_match(buffer, cursor, start, only);
            Vec::new()
        }
        many => {
            if let Some(shared) = common_prefix(many) {
                if shared.len() > prefix.len() {
                    apply_match(buffer, cursor, start, &shared);
                }
            }
            many.to_vec()
        }
    }
}

fn token_at(buffer: &str, cursor: usize) -> (usize, String) {
    let bytes = buffer.as_bytes();
    let mut start = cursor.min(bytes.len());
    while start > 0 && !bytes[start - 1].is_ascii_whitespace() {
        start -= 1;
    }
    (start, buffer[start..cursor.min(buffer.len())].to_owned())
}

fn apply_match(buffer: &mut String, cursor: &mut usize, start: usize, value: &str) {
    let end = (*cursor).min(buffer.len());
    buffer.replace_range(start..end, value);
    *cursor = start + value.len();
}

fn collect_matches(prefix: &str) -> Vec<String> {
    let mut out = Vec::new();
    if prefix.contains('/') || prefix.starts_with('.') {
        collect_file_matches(prefix, &mut out);
    } else {
        for name in BUILTINS {
            if name.starts_with(prefix) {
                out.push((*name).to_owned());
            }
        }
        collect_path_commands(prefix, &mut out);
        collect_file_matches(prefix, &mut out);
    }
    out.sort();
    out.dedup();
    out
}

fn common_prefix(items: &[String]) -> Option<String> {
    let first = items.first()?;
    let mut end = first.len();
    for item in &items[1..] {
        end = end.min(
            first
                .chars()
                .zip(item.chars())
                .take_while(|(a, b)| a == b)
                .count(),
        );
    }
    Some(first.chars().take(end).collect())
}
