//! Temporarily suppress history recording for RC / `source`.

use crate::env::ShellEnvironment;

/// Run `f` with [`ShellEnvironment::suppress_history`] set; restore even on unwind.
pub(super) fn with_suppressed<R>(
    env: &mut ShellEnvironment,
    f: impl FnOnce(&mut ShellEnvironment) -> R,
) -> R {
    let prev = env.suppress_history;
    env.suppress_history = true;
    let _guard = Restore {
        env: env as *mut ShellEnvironment,
        prev,
    };
    // `f` uses the exclusive `&mut`; `_guard` only touches `*env` in `Drop` after that ends.
    f(env)
}

/// Restores `suppress_history` when dropped (normal return or unwind).
struct Restore {
    env: *mut ShellEnvironment,
    prev: bool,
}

impl Drop for Restore {
    fn drop(&mut self) {
        // SAFETY: `env` was derived from an exclusive `&mut ShellEnvironment` that
        // outlives this guard. Drop runs after `f` releases that borrow (or while
        // unwinding past it), so this write does not race with another access.
        unsafe {
            (*self.env).suppress_history = self.prev;
        }
    }
}
