//! Bind group stdin (bytes or file) and run the nested list.

use super::super::super::super::io::ExecIo;
use super::super::super::super::list;
use super::super::super::super::redirect::{HeredocState, StdinSource};
use super::super::super::super::stdout_mode::StdoutMode;
use super::super::super::super::CommandResult;
use crate::env::ShellEnvironment;
use crate::parse::CommandList;

use std::io::{self, BufRead, BufReader, Cursor, Write};

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_stdin<I: BufRead, O: Write, E: Write>(
    list: &CommandList<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    heredocs: &mut HeredocState,
    src: StdinSource,
    stdout_file: Option<std::fs::File>,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    match src {
        StdinSource::Bytes(b) => from_bytes(
            list,
            argv,
            shell_env,
            last_status,
            heredocs,
            b,
            stdout_file,
            io,
        ),
        StdinSource::File(f) => from_file(
            list,
            argv,
            shell_env,
            last_status,
            heredocs,
            f,
            stdout_file,
            io,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn from_bytes<I: BufRead, O: Write, E: Write>(
    list: &CommandList<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    heredocs: &mut HeredocState,
    bytes: Vec<u8>,
    stdout_file: Option<std::fs::File>,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    let mut cursor = Cursor::new(bytes);
    with_stdin(
        list,
        argv,
        shell_env,
        last_status,
        heredocs,
        &mut cursor,
        stdout_file,
        io,
    )
}

#[allow(clippy::too_many_arguments)]
fn from_file<I: BufRead, O: Write, E: Write>(
    list: &CommandList<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    heredocs: &mut HeredocState,
    file: std::fs::File,
    stdout_file: Option<std::fs::File>,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    let mut reader = BufReader::new(file);
    with_stdin(
        list,
        argv,
        shell_env,
        last_status,
        heredocs,
        &mut reader,
        stdout_file,
        io,
    )
}

#[allow(clippy::too_many_arguments)]
fn with_stdin<I: BufRead, O: Write, E: Write>(
    list: &CommandList<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    heredocs: &mut HeredocState,
    stdin: &mut impl BufRead,
    stdout_file: Option<std::fs::File>,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    match stdout_file {
        Some(mut file) => {
            let mut nested = ExecIo {
                stdin,
                stdout: &mut file,
                stderr: io.stderr,
                stdout_mode: StdoutMode::Capture,
            };
            list::execute_list_with(list, argv, shell_env, last_status, heredocs, &mut nested)
        }
        None => {
            let mut nested = ExecIo {
                stdin,
                stdout: io.stdout,
                stderr: io.stderr,
                stdout_mode: io.stdout_mode,
            };
            list::execute_list_with(list, argv, shell_env, last_status, heredocs, &mut nested)
        }
    }
}
