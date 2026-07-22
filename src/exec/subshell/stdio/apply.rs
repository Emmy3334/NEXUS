//! Match on opened group redirect files and run the nested list.

mod stdin;

use super::super::super::io::ExecIo;
use super::super::super::list;
use super::super::super::redirect::{HeredocState, RedirectFiles};
use super::super::super::stdout_mode::StdoutMode;
use super::super::super::CommandResult;
use crate::env::ShellEnvironment;
use crate::parse::CommandList;

use std::io::{self, BufRead, Write};

pub(in crate::exec::subshell) fn apply_group_stdio<I: BufRead, O: Write, E: Write>(
    list: &CommandList<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    heredocs: &mut HeredocState,
    files: RedirectFiles,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    match (files.stdin, files.stdout) {
        (None, None) => list::execute_list_with(list, argv, shell_env, last_status, heredocs, io),
        (None, Some(mut file)) => {
            capture_out(list, argv, shell_env, last_status, heredocs, io, &mut file)
        }
        (Some(src), out) => {
            stdin::bind_stdin(list, argv, shell_env, last_status, heredocs, src, out, io)
        }
    }
}

fn capture_out<I: BufRead, O: Write, E: Write>(
    list: &CommandList<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    heredocs: &mut HeredocState,
    io: &mut ExecIo<'_, I, O, E>,
    file: &mut std::fs::File,
) -> io::Result<CommandResult> {
    let mut nested = ExecIo {
        stdin: io.stdin,
        stdout: file,
        stderr: io.stderr,
        stdout_mode: StdoutMode::Capture,
    };
    list::execute_list_with(list, argv, shell_env, last_status, heredocs, &mut nested)
}
