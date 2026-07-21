//! Running a `( list )` stage inside a `|` pipeline.

mod invoke;

use self::invoke::{call_parent, call_piped, feed_capture};
use super::state::PipeState;
use super::StageCtx;
use crate::exec::{wait_children, CommandResult};
use crate::parse::{CommandList, Redirect};

use std::io::{self, BufRead, Cursor, Write};

/// Run a subshell stage. Returns `Some` when this was the last stage (or hard fail).
pub(super) fn run_subshell_stage<I: BufRead, O: Write, E: Write>(
    list: &CommandList<'_>,
    redirects: &[Redirect<'_>],
    is_last: bool,
    ctx: &mut StageCtx<'_, I, O, E>,
    state: &mut PipeState,
    argv_scratch: &mut Vec<String>,
) -> io::Result<Option<CommandResult>> {
    let pipe_in = take_pipe_bytes(state);
    if is_last {
        let result = run_last(list, redirects, pipe_in, ctx, argv_scratch)?;
        let _ = wait_children(&mut state.children)?;
        return Ok(Some(result));
    }
    let mut buffer = Vec::new();
    let _ = feed_capture(list, redirects, pipe_in, ctx, &mut buffer, argv_scratch)?;
    state.buffered_out = Some(buffer);
    Ok(None)
}

fn take_pipe_bytes(state: &mut PipeState) -> Option<Vec<u8>> {
    if let Some(buffer) = state.buffered_out.take() {
        return Some(buffer);
    }
    if let Some(mut reader) = state.prev_stdout.take() {
        let mut buffer = Vec::new();
        let _ = io::copy(&mut reader, &mut buffer);
        return Some(buffer);
    }
    None
}

fn run_last<I: BufRead, O: Write, E: Write>(
    list: &CommandList<'_>,
    redirects: &[Redirect<'_>],
    pipe_in: Option<Vec<u8>>,
    ctx: &mut StageCtx<'_, I, O, E>,
    argv_scratch: &mut Vec<String>,
) -> io::Result<CommandResult> {
    match pipe_in {
        Some(bytes) => {
            let mut cursor = Cursor::new(bytes);
            call_piped(
                list,
                redirects,
                &mut cursor,
                ctx.stdout_mode,
                ctx,
                argv_scratch,
            )
        }
        None => call_parent(list, redirects, ctx.stdout_mode, ctx, argv_scratch),
    }
}
