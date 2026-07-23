//! Gather command-name candidates from PATH, cwd, and history.

use crate::env::ShellEnvironment;
use crate::history::History;

use std::fs;
use std::path::Path;

const MAX_CANDIDATES: usize = 4096;

pub(super) fn candidates(shell_env: &ShellEnvironment) -> Vec<String> {
    let mut out = Vec::new();
    let path = shell_env.get("PATH").unwrap_or("");
    push_path_names(path, &mut out);
    if let Ok(cwd) = std::env::current_dir() {
        scan_dir(&cwd, &mut out);
    }
    push_history_names(&shell_env.history, &mut out);
    out.sort();
    out.dedup();
    out.truncate(MAX_CANDIDATES);
    out
}

fn push_path_names(path_var: &str, out: &mut Vec<String>) {
    for dir in path_var.split(':').filter(|d| !d.is_empty()) {
        if out.len() >= MAX_CANDIDATES {
            break;
        }
        scan_dir(Path::new(dir), out);
    }
}

fn push_history_names(history: &History, out: &mut Vec<String>) {
    for (_, entry) in history.iter().rev().take(200) {
        if let Some(cmd) = entry.line.split_whitespace().next() {
            if !cmd.contains('/') {
                out.push(cmd.to_owned());
            }
        }
        if out.len() >= MAX_CANDIDATES {
            break;
        }
    }
}

fn scan_dir(dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if out.len() >= MAX_CANDIDATES {
            break;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if meta.permissions().mode() & 0o111 == 0 {
                continue;
            }
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.contains('/') {
            out.push(name);
        }
    }
}
