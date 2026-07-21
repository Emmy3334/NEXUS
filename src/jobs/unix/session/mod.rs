//! Interactive shell signal disposition.

mod tty;

use nix::sys::signal::{signal, SigHandler, Signal};
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};

pub(in crate::jobs::unix) use tty::{give_terminal, take_terminal};

static ENABLED: AtomicBool = AtomicBool::new(false);

/// Whether this session should use process groups + terminal handover.
#[must_use]
pub(crate) fn job_control_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Ignore job-control signals in the shell so Ctrl-C/Z do not kill/stop it.
pub(crate) fn install_interactive_handlers() -> io::Result<()> {
    if !tty::stdin_is_tty() {
        return Ok(());
    }
    ignore(Signal::SIGINT)?;
    ignore(Signal::SIGQUIT)?;
    ignore(Signal::SIGTSTP)?;
    ignore(Signal::SIGTTIN)?;
    ignore(Signal::SIGTTOU)?;
    tty::claim_for_shell()?;
    ENABLED.store(true, Ordering::Relaxed);
    Ok(())
}

fn ignore(sig: Signal) -> io::Result<()> {
    // SAFETY: only installing SIG_IGN, never a custom handler pointer.
    unsafe { signal(sig, SigHandler::SigIgn) }
        .map(|_| ())
        .map_err(tty::nix_err)
}
