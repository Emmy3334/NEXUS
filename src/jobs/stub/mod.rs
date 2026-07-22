//! Non-Unix fallbacks: no process-group or terminal job control.

mod child;
mod wait;

pub(crate) use child::{
    assign_process_group, detach_background, prepare_background_group, prepare_child_command,
    prepare_process_group,
};
pub(crate) use wait::{
    continue_background, install_interactive_handlers, job_control_enabled, sigtstp_status,
    wait_foreground, wait_pipeline, FgWait,
};
