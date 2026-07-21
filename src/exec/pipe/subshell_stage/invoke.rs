//! Invoke helpers for subshell pipeline stages.

use super::super::StageCtx;
use crate::exec::io::ExecIo;
use crate::exec::stdout_mode::StdoutMode;
use crate::exec::subshell;
use crate::exec::CommandResult;
use crate::parse::{CommandList, Redirect};

use std::io::{self, BufRead, Cursor, Write};

pub(super) fn feed_capture<I: BufRead, O: Write, E: Write>(
    list: &CommandList<'_>,
    redirects: &[Redirect<'_>],
    pipe_in: Option<Vec<u8>>,
    ctx: &mut StageCtx<'_, I, O, E>,
    buffer: &mut Vec<u8>,
    argv_scratch: &mut Vec<String>,
) -> io::Result<CommandResult> {
    match pipe_in {
        Some(bytes) => {
            let mut cursor = Cursor::new(bytes);
            call_out(list, redirects, &mut cursor, buffer, ctx, argv_scratch)
        }
        None => capture_from_parent(list, redirects, ctx, buffer, argv_scratch),
    }
}

pub(super) fn call_piped<I: BufRead, O: Write, E: Write>(
    list: &CommandList<'_>,
    redirects: &[Redirect<'_>],
    stdin: &mut impl BufRead,
    mode: StdoutMode,
    ctx: &mut StageCtx<'_, I, O, E>,
    argv_scratch: &mut Vec<String>,
) -> io::Result<CommandResult> {
    let mut nested = ExecIo {
        stdin,
        stdout: ctx.stdout,
        stderr: ctx.stderr,
        stdout_mode: mode,
    };
    subshell::run_subshell(
        list,
        redirects,
        argv_scratch,
        ctx.shell_env,
        ctx.last_status,
        ctx.heredocs,
        &mut nested,
    )
}

pub(super) fn call_parent<I: BufRead, O: Write, E: Write>(
    list: &CommandList<'_>,
    redirects: &[Redirect<'_>],
    mode: StdoutMode,
    ctx: &mut StageCtx<'_, I, O, E>,
    argv_scratch: &mut Vec<String>,
) -> io::Result<CommandResult> {
    let mut nested = ExecIo {
        stdin: ctx.stdin,
        stdout: ctx.stdout,
        stderr: ctx.stderr,
        stdout_mode: mode,
    };
    subshell::run_subshell(
        list,
        redirects,
        argv_scratch,
        ctx.shell_env,
        ctx.last_status,
        ctx.heredocs,
        &mut nested,
    )
}

fn call_out<I: BufRead, O: Write, E: Write>(
    list: &CommandList<'_>,
    redirects: &[Redirect<'_>],
    stdin: &mut impl BufRead,
    stdout: &mut impl Write,
    ctx: &mut StageCtx<'_, I, O, E>,
    argv_scratch: &mut Vec<String>,
) -> io::Result<CommandResult> {
    let mut nested = ExecIo {
        stdin,
        stdout,
        stderr: ctx.stderr,
        stdout_mode: StdoutMode::Capture,
    };
    subshell::run_subshell(
        list,
        redirects,
        argv_scratch,
        ctx.shell_env,
        ctx.last_status,
        ctx.heredocs,
        &mut nested,
    )
}

fn capture_from_parent<I: BufRead, O: Write, E: Write>(
    list: &CommandList<'_>,
    redirects: &[Redirect<'_>],
    ctx: &mut StageCtx<'_, I, O, E>,
    buffer: &mut Vec<u8>,
    argv_scratch: &mut Vec<String>,
) -> io::Result<CommandResult> {
    let mut nested = ExecIo {
        stdin: ctx.stdin,
        stdout: buffer,
        stderr: ctx.stderr,
        stdout_mode: StdoutMode::Capture,
    };
    subshell::run_subshell(
        list,
        redirects,
        argv_scratch,
        ctx.shell_env,
        ctx.last_status,
        ctx.heredocs,
        &mut nested,
    )
}
