//! Local git branch names from `.git/refs/heads/`.

use std::fs;
use std::path::Path;

pub(super) fn collect_branches(prefix: &str, out: &mut Vec<String>) {
    walk(Path::new(".git/refs/heads"), "", prefix, out);
}

fn walk(dir: &Path, relative: &str, prefix: &str, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let branch = if relative.is_empty() {
            name.clone()
        } else {
            format!("{relative}/{name}")
        };
        let path = entry.path();
        if path.is_dir() {
            walk(&path, &branch, prefix, out);
        } else if branch.starts_with(prefix) {
            out.push(branch);
        }
    }
}
