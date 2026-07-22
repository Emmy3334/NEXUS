//! Terminal ownership (`tcsetpgrp`) for foreground jobs.

use nix::unistd::{getpgrp, isatty, tcsetpgrp, Pid};
use std::io;
use std::os::fd::{BorrowedFd, RawFd};

use super::job_control_enabled;

const STDIN_FD: RawFd = 0;

pub(in crate::jobs::unix) fn give_terminal(pgid: Pid) -> io::Result<()> {
    if !job_control_enabled() {
        return Ok(());
    }
    tcsetpgrp(stdin_fd()?, pgid).map_err(nix_err)
}

pub(in crate::jobs::unix) fn take_terminal() -> io::Result<()> {
    if !job_control_enabled() {
        return Ok(());
    }
    tcsetpgrp(stdin_fd()?, getpgrp()).map_err(nix_err)
}

pub(super) fn claim_for_shell() -> io::Result<()> {
    tcsetpgrp(stdin_fd()?, getpgrp()).map_err(nix_err)
}

pub(super) fn stdin_is_tty() -> bool {
    isatty(STDIN_FD).unwrap_or(false)
}

pub(super) fn nix_err(err: nix::Error) -> io::Error {
    io::Error::from_raw_os_error(err as i32)
}

fn stdin_fd() -> io::Result<BorrowedFd<'static>> {
    // SAFETY: fd 0 remains open for the process lifetime we care about.
    Ok(unsafe { BorrowedFd::borrow_raw(STDIN_FD) })
}
