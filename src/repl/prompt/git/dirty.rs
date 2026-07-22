//! Dirty work tree detection via `git status --porcelain`, with a short cache.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};

const TTL: Duration = Duration::from_millis(750);

struct Cache {
    git_dir: PathBuf,
    stamp: SystemTime,
    checked_at: Instant,
    dirty: bool,
}

static CACHE: Mutex<Option<Cache>> = Mutex::new(None);

/// True when the work tree is dirty; false if clean or `git` is unavailable.
pub(super) fn is_dirty(git_dir: &Path) -> bool {
    let stamp = meta_stamp(git_dir).unwrap_or(SystemTime::UNIX_EPOCH);
    if let Some(hit) = cache_hit(git_dir, stamp) {
        return hit;
    }
    let dirty = query_porcelain();
    store(git_dir, stamp, dirty);
    dirty
}

fn cache_hit(git_dir: &Path, stamp: SystemTime) -> Option<bool> {
    let guard = CACHE.lock().ok()?;
    let cache = guard.as_ref()?;
    if cache.git_dir == git_dir && cache.stamp == stamp && cache.checked_at.elapsed() < TTL {
        Some(cache.dirty)
    } else {
        None
    }
}

fn store(git_dir: &Path, stamp: SystemTime, dirty: bool) {
    let Ok(mut guard) = CACHE.lock() else {
        return;
    };
    *guard = Some(Cache {
        git_dir: git_dir.to_path_buf(),
        stamp,
        checked_at: Instant::now(),
        dirty,
    });
}

fn meta_stamp(git_dir: &Path) -> Option<SystemTime> {
    let head = mtime(&git_dir.join("HEAD"))?;
    match mtime(&git_dir.join("index")) {
        Some(index) if index > head => Some(index),
        _ => Some(head),
    }
}

fn mtime(path: &Path) -> Option<SystemTime> {
    fs_meta(path).and_then(|m| m.modified().ok())
}

fn fs_meta(path: &Path) -> Option<std::fs::Metadata> {
    std::fs::metadata(path).ok()
}

fn query_porcelain() -> bool {
    let output = Command::new("git")
        .args(["status", "--porcelain", "-uno"])
        .output();
    match output {
        Ok(out) if out.status.success() => !out.stdout.is_empty(),
        _ => false,
    }
}
