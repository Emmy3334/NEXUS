//! Save / restore process working directory (subshell & command substitution).

use std::cell::Cell;
use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

static CWD_SCOPE: Mutex<()> = Mutex::new(());

thread_local! {
    static CWD_SCOPE_DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// Serialize subshell / command-substitution cwd changes (reentrant on one thread).
pub(crate) struct CwdScope {
    _serial: Option<MutexGuard<'static, ()>>,
}

impl CwdScope {
    pub(crate) fn new() -> Self {
        let depth = CWD_SCOPE_DEPTH.with(|cell| {
            let depth = cell.get();
            cell.set(depth + 1);
            depth
        });
        let serial = if depth == 0 {
            Some(CWD_SCOPE.lock().unwrap_or_else(PoisonError::into_inner))
        } else {
            None
        };
        Self { _serial: serial }
    }
}

impl Drop for CwdScope {
    fn drop(&mut self) {
        CWD_SCOPE_DEPTH.with(|cell| cell.set(cell.get().saturating_sub(1)));
    }
}

/// Snapshot the current directory if it can be read.
#[must_use]
pub(crate) fn save() -> Option<PathBuf> {
    env::current_dir().ok()
}

/// Restore `saved` cwd; log failures to `stderr` without aborting.
pub(crate) fn restore(saved: Option<PathBuf>, stderr: &mut impl Write) {
    if let Some(path) = saved {
        if let Err(err) = env::set_current_dir(&path) {
            let _ = writeln!(stderr, "nexus: failed to restore cwd: {err}");
        }
    }
}

/// RAII guard that restores the saved cwd on drop (including on unwind).
pub(crate) struct CwdGuard {
    saved: Option<PathBuf>,
}

impl CwdGuard {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self { saved: save() }
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        if let Some(path) = self.saved.take() {
            let _ = env::set_current_dir(path);
        }
    }
}
