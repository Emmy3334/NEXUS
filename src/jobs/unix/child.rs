//! Child `pre_exec`: restore default signal disposition so Ctrl-C/Z reach jobs.

use nix::sys::signal::{signal, SigHandler, Signal};
use nix::unistd::{setpgid, Pid};
use std::os::unix::process::CommandExt;
use std::process::Command;

/// Prepare `command` so children do not inherit the shell's SIG_IGN handlers.
pub(crate) fn prepare_child_command(command: &mut Command) {
    // SAFETY: only async-signal-safe `signal(SIG_DFL)` in the child.
    unsafe {
        command.pre_exec(|| {
            restore_default(Signal::SIGINT)?;
            restore_default(Signal::SIGQUIT)?;
            restore_default(Signal::SIGTSTP)?;
            restore_default(Signal::SIGTTIN)?;
            restore_default(Signal::SIGTTOU)?;
            Ok(())
        });
    }
}

/// Place a spawned command in `pgid`, or make it a new group leader.
pub(crate) fn prepare_process_group(command: &mut Command, pgid: Option<i32>) {
    if !super::session::job_control_enabled() {
        return;
    }
    // SAFETY: `setpgid` is async-signal-safe between fork and exec.
    unsafe {
        command.pre_exec(move || {
            let group = Pid::from_raw(pgid.unwrap_or(0));
            setpgid(Pid::from_raw(0), group).map_err(nix_error)
        });
    }
}

/// Make a background wrapper process its own process-group leader.
pub(crate) fn prepare_background_group(command: &mut Command) {
    // SAFETY: `setpgid` is async-signal-safe between fork and exec.
    unsafe {
        command.pre_exec(|| setpgid(Pid::from_raw(0), Pid::from_raw(0)).map_err(nix_error));
    }
}

/// Confirm the child's process group in the parent, closing the fork race.
pub(crate) fn assign_process_group(pid: u32, pgid: Option<i32>) -> i32 {
    let pid = Pid::from_raw(pid as i32);
    let group = Pid::from_raw(pgid.unwrap_or(pid.as_raw()));
    let _ = setpgid(pid, group);
    group.as_raw()
}

fn restore_default(sig: Signal) -> Result<(), std::io::Error> {
    // SAFETY: installing SIG_DFL only.
    unsafe { signal(sig, SigHandler::SigDfl) }
        .map(|_| ())
        .map_err(|err| std::io::Error::from_raw_os_error(err as i32))
}

fn nix_error(err: nix::Error) -> std::io::Error {
    std::io::Error::from_raw_os_error(err as i32)
}
