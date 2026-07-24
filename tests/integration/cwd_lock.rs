//! Process-wide lock for tests that mutate `std::env::current_dir`.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

static CWD_LOCK: Mutex<()> = Mutex::new(());

/// Acquire the shared cwd lock (poison-tolerant).
pub fn lock() -> MutexGuard<'static, ()> {
    CWD_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Hold the cwd lock and restore the entry directory on drop (including on panic).
pub struct RestoreCwd {
    start: PathBuf,
    _lock: MutexGuard<'static, ()>,
}

impl RestoreCwd {
    /// Snapshot cwd under the global lock.
    pub fn new() -> Self {
        let lock = lock();
        let start = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self { start, _lock: lock }
    }

    /// Directory captured when the guard was created.
    pub fn start(&self) -> &Path {
        &self.start
    }

    /// Change cwd while the lock is held.
    pub fn chdir(&self, path: &Path) -> std::io::Result<()> {
        std::env::set_current_dir(path)
    }
}

impl Drop for RestoreCwd {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.start);
    }
}
