//! Builtin / function execution with redirect file handles.

use super::files::RedirectFiles;
use crate::builtins;
use crate::env::ShellEnvironment;
use crate::exec::io::ExecIo;
use crate::exec::{execute_command_mode, CommandResult};

use std::io::{self, BufRead, Write};

pub(super) fn run_function<I: BufRead, O: Write, E: Write>(
    argv: &[String],
    files: RedirectFiles,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    drop(files.stdin);
    match files.stdout {
        Some(mut file) => {
            Ok(
                crate::functions::try_run(argv, shell_env, last_status, &mut file, io.stderr)?
                    .unwrap_or(CommandResult::Status(0)),
            )
        }
        None => execute_command_mode(
            io.stdout_mode,
            argv,
            shell_env,
            last_status,
            io.stdout,
            io.stderr,
        ),
    }
}

pub(super) fn run_builtin<I: BufRead, O: Write, E: Write>(
    argv: &[String],
    files: RedirectFiles,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    drop(files.stdin);
    let result = match files.stdout {
        Some(mut file) => builtins::try_run(argv, shell_env, last_status, &mut file, io.stderr)?,
        None => builtins::try_run(argv, shell_env, last_status, io.stdout, io.stderr)?,
    };
    Ok(result
        .map(CommandResult::from)
        .unwrap_or(CommandResult::Status(0)))
}
