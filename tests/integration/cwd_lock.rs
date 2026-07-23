//! Process-wide lock for tests that mutate `std::env::current_dir`.

use std::sync::{Mutex, MutexGuard};

static CWD_LOCK: Mutex<()> = Mutex::new(());

/// Acquire the shared cwd lock (poison-tolerant).
pub fn lock() -> MutexGuard<'static, ()> {
    CWD_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
