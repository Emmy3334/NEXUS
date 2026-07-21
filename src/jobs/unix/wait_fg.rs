//! Foreground wait with `WUNTRACED` (Ctrl-Z) and `fg`/`bg` continue.

use super::session::{give_terminal, take_terminal};
use nix::sys::signal::{killpg, Signal};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{setpgid, Pid};
use std::io::{self, Write};
use std::process::Child;

/// Result of waiting on a foreground process group leader.
#[derive(Debug)]
pub(crate) enum FgWait {
    /// Process exited or was killed.
    Done(u8),
    /// Process stopped (e.g. Ctrl-Z / SIGTSTP).
    Stopped,
}

/// Wait for `child` in the foreground; place it in its own process group.
pub(crate) fn wait_foreground(child: &mut Child) -> io::Result<FgWait> {
    let pid = Pid::from_raw(child.id() as i32);
    let _ = setpgid(pid, pid);
    let outcome = wait_pipeline(std::slice::from_mut(child), pid.as_raw())?;
    if matches!(outcome, FgWait::Done(_)) {
        let _ = child.try_wait();
    }
    Ok(outcome)
}

/// Put `child` in its own process group without waiting (for `&`).
pub(crate) fn detach_background(child: &Child) -> io::Result<i32> {
    let pid = Pid::from_raw(child.id() as i32);
    let _ = setpgid(pid, pid);
    Ok(pid.as_raw())
}

/// Wait for every member of a foreground pipeline process group.
pub(crate) fn wait_pipeline(children: &mut [Child], pgid: i32) -> io::Result<FgWait> {
    if children.is_empty() {
        return Ok(FgWait::Done(0));
    }
    give_terminal(Pid::from_raw(pgid))?;
    let target = Pid::from_raw(-pgid);
    let last_pid = children.last().map(Child::id).unwrap_or(0);
    let outcome = wait_until_done_or_stopped(target, children.len(), last_pid);
    let restore = take_terminal();
    let outcome = outcome?;
    restore?;
    Ok(outcome)
}

/// Send `SIGCONT` without taking the terminal (for `bg`).
pub(crate) fn continue_background(pgid: i32) -> io::Result<()> {
    killpg(Pid::from_raw(pgid), Signal::SIGCONT)
        .map_err(|err| io::Error::from_raw_os_error(err as i32))
}

fn wait_until_done_or_stopped(pid: Pid, mut remaining: usize, last_pid: u32) -> io::Result<FgWait> {
    let mut last_status = 0;
    let mut noted_interrupt = false;
    while remaining > 0 {
        match waitpid(pid, Some(WaitPidFlag::WUNTRACED)).map_err(nix_err)? {
            WaitStatus::Exited(child, code) => {
                remaining -= 1;
                if child.as_raw() as u32 == last_pid {
                    last_status = code as u8;
                }
            }
            WaitStatus::Signaled(child, sig, _) => {
                remaining -= 1;
                if sig == Signal::SIGINT && !noted_interrupt {
                    // Terminal echoed `^C` without a newline; land the prompt cleanly.
                    let _ = writeln!(io::stderr());
                    noted_interrupt = true;
                }
                if child.as_raw() as u32 == last_pid {
                    last_status = 128u8.saturating_add(sig as u8);
                }
            }
            WaitStatus::Stopped(_, _) => return Ok(FgWait::Stopped),
            WaitStatus::Continued(_) => continue,
            WaitStatus::StillAlive => continue,
            #[allow(unreachable_patterns)]
            _ => continue,
        }
    }
    Ok(FgWait::Done(last_status))
}

fn nix_err(err: nix::Error) -> io::Error {
    io::Error::from_raw_os_error(err as i32)
}
