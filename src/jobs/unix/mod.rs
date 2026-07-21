//! Unix process-group / signal helpers for interactive job control.

mod child;
mod session;
mod wait_fg;

pub(crate) use child::{
    assign_process_group, prepare_background_group, prepare_child_command, prepare_process_group,
};
pub(crate) use session::{install_interactive_handlers, job_control_enabled};
pub(crate) use wait_fg::{
    continue_background, detach_background, wait_foreground, wait_pipeline, FgWait,
};
