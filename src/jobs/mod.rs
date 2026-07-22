//! Background job table for `&` / `jobs` / `fg` / `bg`.

#[cfg(unix)]
mod unix;

mod table;

pub use table::{JobSpec, JobState, JobTable, JobTableError};

/// Whether interactive job control (pgrp / tty) is active for this process.
#[must_use]
pub fn job_control_active() -> bool {
    job_control_enabled()
}

#[cfg(unix)]
pub(crate) use unix::{
    assign_process_group, continue_background, detach_background, install_interactive_handlers,
    job_control_enabled, prepare_background_group, prepare_child_command, prepare_process_group,
    wait_foreground, wait_pipeline, FgWait,
};

#[cfg(not(unix))]
mod stub;

#[cfg(not(unix))]
pub(crate) use stub::*;

#[cfg(unix)]
pub(crate) fn sigtstp_status() -> u8 {
    128u8.saturating_add(nix::sys::signal::Signal::SIGTSTP as u8)
}
