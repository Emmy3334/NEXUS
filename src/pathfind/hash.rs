//! Command-path hash and PATH directory listing cache (zsh HASHCMDS analogue).

use super::is_executable;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

struct PathCache {
    commands: HashMap<String, PathBuf>,
    dir_names: HashMap<PathBuf, Vec<String>>,
}

struct Cache {
    paths: HashMap<String, PathCache>,
}

static CACHE: LazyLock<Mutex<Cache>> = LazyLock::new(|| {
    Mutex::new(Cache {
        paths: HashMap::new(),
    })
});

fn with_path<R>(path_var: &str, f: impl FnOnce(&mut PathCache) -> R) -> R {
    let mut guard = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let entry = guard
        .paths
        .entry(path_var.to_owned())
        .or_insert_with(|| PathCache {
            commands: HashMap::new(),
            dir_names: HashMap::new(),
        });
    f(entry)
}

/// Cached hit when still executable; else miss.
#[must_use]
pub fn lookup(name: &str, path_var: &str) -> Option<PathBuf> {
    with_path(path_var, |cache| {
        let path = cache.commands.get(name)?.clone();
        if is_executable(&path).is_some() {
            Some(path)
        } else {
            cache.commands.remove(name);
            None
        }
    })
}

pub fn remember(name: &str, path_var: &str, path: PathBuf) {
    with_path(path_var, |cache| {
        cache.commands.insert(name.to_owned(), path);
    });
}

/// Drop all cached command paths and directory listings.
pub fn clear() {
    let mut guard = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    guard.paths.clear();
}

/// Sorted `(name, path)` pairs currently hashed for `path_var`.
#[must_use]
pub fn entries(path_var: &str) -> Vec<(String, PathBuf)> {
    with_path(path_var, |cache| {
        let mut out: Vec<_> = cache
            .commands
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    })
}

/// PATH command names matching `prefix`, using cached directory scans.
#[must_use]
pub fn list_commands(prefix: &str, path_var: &str) -> Vec<String> {
    with_path(path_var, |cache| {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for dir in path_var.split(':').filter(|d| !d.is_empty()) {
            let dir_path = PathBuf::from(dir);
            for name in dir_names(cache, &dir_path) {
                if !prefix.is_empty()
                    && !name.starts_with(prefix)
                    && !name
                        .to_ascii_lowercase()
                        .starts_with(&prefix.to_ascii_lowercase())
                {
                    continue;
                }
                if !seen.insert(name.clone()) {
                    continue;
                }
                out.push(name);
            }
        }
        out.sort();
        out
    })
}

fn dir_names(cache: &mut PathCache, dir: &Path) -> Vec<String> {
    if !cache.dir_names.contains_key(dir) {
        let mut names = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if is_executable(&path).is_some() {
                    names.push(entry.file_name().to_string_lossy().into_owned());
                }
            }
            names.sort();
        }
        cache.dir_names.insert(dir.to_path_buf(), names);
    }
    cache.dir_names.get(dir).cloned().unwrap_or_default()
}
