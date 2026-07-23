//! No-op process-group preparation on non-Unix targets.

use crate::env::ShellEnvironment;
use std::io;
use std::process::{Child, Command};

pub(crate) fn prepare_child_command(_command: &mut Command, _env: &ShellEnvironment) {}

pub(crate) fn prepare_background_group(_command: &mut Command, _env: &ShellEnvironment) {}

pub(crate) fn prepare_process_group(
    _command: &mut Command,
    _pgid: Option<i32>,
    _env: &ShellEnvironment,
) {
}

pub(crate) fn assign_process_group(pid: u32, pgid: Option<i32>) -> i32 {
    pgid.unwrap_or(pid as i32)
}

pub(crate) fn detach_background(child: &Child) -> io::Result<i32> {
    Ok(child.id() as i32)
}
