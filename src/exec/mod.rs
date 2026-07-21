//! Command execution: builtins first, then external programs.
//!
//! - [`execute_list`] walks `;`-separated pipelines
//! - [`pipe`] connects stages with OS pipes
//! - [`redirect`] applies `<` / `>` / `>>` / `<<` (overrides a pipe on that fd)
//! - [`process`] holds the child-process helpers shared by the above

mod capture;
mod command;
mod io;
mod list;
mod pipe;
mod process;
mod redirect;
mod stdout_mode;
mod subshell;

pub(crate) use capture::capture_command_output;
pub(crate) use command::execute_command_mode;
pub use command::{execute_command, execute_external};
pub use list::execute_list;
pub(crate) use list::execute_list_captured;
pub use redirect::collect_heredoc_bodies;
pub(crate) use stdout_mode::StdoutMode;

pub(crate) use process::{
    abandon_children, build_external_command, exit_status_code, report_spawn_failure,
    run_builtin_status, wait_children,
};

use crate::builtins;

/// Outcome of running one simple command or a list/pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use = "shell exit vs continue must be handled by the REPL"]
pub enum CommandResult {
    /// Keep the REPL running with this status.
    Status(u8),
    /// Terminate the shell with this status (`exit` builtin).
    Exit(u8),
}

impl From<builtins::BuiltinResult> for CommandResult {
    fn from(result: builtins::BuiltinResult) -> Self {
        match result {
            builtins::BuiltinResult::Status(code) => Self::Status(code),
            builtins::BuiltinResult::Exit(code) => Self::Exit(code),
        }
    }
}
