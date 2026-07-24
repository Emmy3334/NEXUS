//! Filesystem / PATH scanners for completion.

use super::matchers::matches_prefix;
use crate::pathfind;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn collect_path_commands(prefix: &str, out: &mut Vec<String>) {
    let Ok(path) = std::env::var("PATH") else {
        return;
    };
    for name in pathfind::list_commands("", &path) {
        if matches_prefix(&name, prefix) {
            out.push(name);
        }
    }
}

pub(super) fn collect_file_matches(prefix: &str, out: &mut Vec<String>) {
    let path = Path::new(prefix);
    let (dir, file_prefix) = match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) if !prefix.is_empty() => {
            (parent_or_dot(parent), name.to_string_lossy().into_owned())
        }
        _ => (PathBuf::from("."), prefix.to_owned()),
    };
    let Ok(entries) = fs::read_dir(&dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !matches_prefix(&name, &file_prefix) {
            continue;
        }
        let mut rendered = dir.join(&name).display().to_string();
        if entry.path().is_dir() {
            rendered.push('/');
        }
        out.push(strip_dot_slash(&rendered));
    }
}

fn parent_or_dot(parent: &Path) -> PathBuf {
    if parent.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        parent.to_path_buf()
    }
}

fn strip_dot_slash(path: &str) -> String {
    path.strip_prefix("./").unwrap_or(path).to_owned()
}
