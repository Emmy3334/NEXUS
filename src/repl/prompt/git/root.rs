//! Locate `.git` walking up from the process cwd.

use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn find_git_dir() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let candidate = dir.join(".git");
        if let Some(git_dir) = resolve_git_dir(&candidate) {
            return Some(git_dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn resolve_git_dir(path: &Path) -> Option<PathBuf> {
    if path.is_dir() {
        return Some(path.to_path_buf());
    }
    let contents = fs::read_to_string(path).ok()?;
    let line = contents.lines().next()?.trim();
    let gitdir = line.strip_prefix("gitdir:")?.trim();
    let resolved = if Path::new(gitdir).is_absolute() {
        PathBuf::from(gitdir)
    } else {
        path.parent()?.join(gitdir)
    };
    resolved.is_dir().then_some(resolved)
}
