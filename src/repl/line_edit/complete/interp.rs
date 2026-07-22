//! Interpreter-aware file completion (e.g. `python` → `*.py`).

use super::paths::collect_file_matches;

pub(super) fn collect(prefix: &str, extensions: &[&str], out: &mut Vec<String>) {
    let mut raw = Vec::new();
    collect_file_matches(prefix, &mut raw);
    for path in raw {
        if keep(&path, extensions) {
            out.push(path);
        }
    }
}

fn keep(path: &str, extensions: &[&str]) -> bool {
    if path.ends_with('/') {
        return true;
    }
    extensions.iter().any(|ext| path.ends_with(ext))
}
