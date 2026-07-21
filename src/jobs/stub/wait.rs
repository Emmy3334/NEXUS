//! Ordinary child waits on platforms without Unix job control.

use std::io;
use std::process::Child;

pub(crate) enum FgWait {
    Done(u8),
    Stopped,
}

pub(crate) fn install_interactive_handlers() -> io::Result<()> {
    Ok(())
}

pub(crate) fn job_control_enabled() -> bool {
    false
}

pub(crate) fn wait_foreground(child: &mut Child) -> io::Result<FgWait> {
    Ok(FgWait::Done(crate::exec::exit_status_code(child.wait()?)))
}

pub(crate) fn wait_pipeline(children: &mut [Child], _pgid: i32) -> io::Result<FgWait> {
    let mut status = 0;
    for child in children {
        status = crate::exec::exit_status_code(child.wait()?);
    }
    Ok(FgWait::Done(status))
}

pub(crate) fn continue_background(_pgid: i32) -> io::Result<()> {
    Ok(())
}

pub(crate) fn sigtstp_status() -> u8 {
    148
}
