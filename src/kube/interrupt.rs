//! Temporary SIGINT cancel for long-running kube streams (`logs -f` / `exec`).

use nix::sys::signal::{signal, SigHandler, Signal};
use std::sync::atomic::{AtomicBool, Ordering};

static CANCEL: AtomicBool = AtomicBool::new(false);

extern "C" fn on_sigint(_: nix::libc::c_int) {
    CANCEL.store(true, Ordering::SeqCst);
}

/// Whether Ctrl-C was received during [`with_sigint_cancel`].
#[must_use]
pub(super) fn cancelled() -> bool {
    CANCEL.load(Ordering::SeqCst)
}

/// Run `f` with a SIGINT handler that sets the cancel flag, then restore disposition.
pub(super) fn with_sigint_cancel<R>(f: impl FnOnce() -> R) -> R {
    CANCEL.store(false, Ordering::SeqCst);
    // SAFETY: handler only stores an AtomicBool (async-signal-safe).
    let _ = unsafe { signal(Signal::SIGINT, SigHandler::Handler(on_sigint)) };
    let out = f();
    restore_sigint();
    CANCEL.store(false, Ordering::SeqCst);
    out
}

fn restore_sigint() {
    let handler = if crate::jobs::job_control_enabled() {
        SigHandler::SigIgn
    } else {
        SigHandler::SigDfl
    };
    let _ = unsafe { signal(Signal::SIGINT, handler) };
}
