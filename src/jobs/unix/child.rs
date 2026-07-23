//! Child `pre_exec`: signals, optional rlimits, optional process group.

use crate::env::ShellEnvironment;
use crate::harden::rlimit::{self, ChildLimits};

use nix::sys::signal::{signal, SigHandler, Signal};
use nix::unistd::{setpgid, Pid};
use std::os::unix::process::CommandExt;
use std::process::Command;

/// Prepare `command` so children restore signals and apply optional rlimits.
pub(crate) fn prepare_child_command(command: &mut Command, env: &ShellEnvironment) {
    install(command, rlimit::limits_from(env), None);
}

/// Place a spawned command in `pgid`, or make it a new group leader (keeps rlimits).
pub(crate) fn prepare_process_group(
    command: &mut Command,
    pgid: Option<i32>,
    env: &ShellEnvironment,
) {
    if !super::session::job_control_enabled() {
        return;
    }
    install(command, rlimit::limits_from(env), Some(pgid));
}

/// Make a background wrapper process its own process-group leader.
pub(crate) fn prepare_background_group(command: &mut Command, env: &ShellEnvironment) {
    install(command, rlimit::limits_from(env), Some(None));
}

/// Confirm the child's process group in the parent, closing the fork race.
pub(crate) fn assign_process_group(pid: u32, pgid: Option<i32>) -> i32 {
    let pid = Pid::from_raw(pid as i32);
    let group = Pid::from_raw(pgid.unwrap_or(pid.as_raw()));
    let _ = setpgid(pid, group);
    group.as_raw()
}

fn install(command: &mut Command, limits: Option<ChildLimits>, pgid: Option<Option<i32>>) {
    // SAFETY: only async-signal-safe calls between fork and exec.
    unsafe {
        command.pre_exec(move || {
            restore_default(Signal::SIGINT)?;
            restore_default(Signal::SIGQUIT)?;
            restore_default(Signal::SIGTSTP)?;
            restore_default(Signal::SIGTTIN)?;
            restore_default(Signal::SIGTTOU)?;
            if let Some(lim) = limits {
                rlimit::apply(&lim)?;
            }
            if let Some(pg) = pgid {
                let group = Pid::from_raw(pg.unwrap_or(0));
                setpgid(Pid::from_raw(0), group).map_err(nix_error)?;
            }
            Ok(())
        });
    }
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
