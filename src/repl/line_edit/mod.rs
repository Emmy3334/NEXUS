//! Physical / logical line acquisition for the REPL.

mod complete;
mod continue_line;
mod input;
mod isearch;
mod plain;
mod probe;
mod recall;
#[cfg(unix)]
mod tty;

pub use crate::keybind::{Action, KeyBindings};
pub use input::ReplInput;
pub use isearch::HistoryISearch;
pub use recall::HistoryRecall;
#[cfg(unix)]
pub use tty::{after_line_down, after_line_up, take_complete_line};

pub use complete::{
    command_names, complete, complete_matches_for_test, complete_or_cycle, CompleteCtx,
    CompleteCycle, Match, Tag,
};
/// Column layout helpers for ambiguous completion listings.
pub use complete::{
    format_columns, list_display_lines, list_display_lines_width, list_menu_fit,
    list_menu_fit_width, list_menu_lines, list_menu_lines_tagged,
};

use super::prompt::{self, PromptContext};
use crate::history::History;
use crate::keybind::KeyBindings as Bindings;
use std::collections::VecDeque;
use std::io::{self, Write};

/// Read one logical command line (may span physical lines when quotes are open).
#[allow(clippy::too_many_arguments)]
pub(super) fn read_logical_line(
    stdin: &mut impl ReplInput,
    stdout: &mut impl Write,
    interactive: bool,
    history: &History,
    bindings: &mut Bindings,
    buffer: &mut String,
    queue: &mut VecDeque<u8>,
    complete_ctx: Option<&CompleteCtx<'_>>,
    prompt_ctx: Option<&PromptContext>,
) -> io::Result<ReadOutcome> {
    buffer.clear();
    if interactive && stdin.is_terminal() {
        let ctx = complete_ctx.expect("TTY line edit requires CompleteCtx");
        return read_tty(stdout, buffer, history, bindings, queue, ctx, prompt_ctx);
    }
    write_prompt(stdout, interactive, prompt_ctx)?;
    if !plain::read_into(stdin, buffer, queue)? {
        return Ok(ReadOutcome::Eof);
    }
    if interactive {
        continue_line::join_until_closed(stdin, stdout, buffer, queue)?;
    }
    Ok(ReadOutcome::Line)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReadOutcome {
    Eof,
    Line,
}

fn write_prompt(
    stdout: &mut impl Write,
    interactive: bool,
    prompt_ctx: Option<&PromptContext>,
) -> io::Result<()> {
    match prompt_ctx {
        Some(ctx) => prompt::write_primary_ctx(stdout, interactive, ctx),
        None => prompt::write_primary(stdout, interactive),
    }
}

fn read_tty(
    stdout: &mut impl Write,
    buffer: &mut String,
    history: &History,
    bindings: &mut Bindings,
    queue: &mut VecDeque<u8>,
    complete_ctx: &CompleteCtx<'_>,
    prompt_ctx: Option<&PromptContext>,
) -> io::Result<ReadOutcome> {
    #[cfg(unix)]
    {
        let owned;
        let ctx = match prompt_ctx {
            Some(c) => c,
            None => {
                owned = PromptContext::classic_default();
                &owned
            }
        };
        tty::edit_line(stdout, buffer, history, bindings, queue, complete_ctx, ctx)
    }
    #[cfg(not(unix))]
    {
        let _ = (
            stdout,
            buffer,
            history,
            bindings,
            queue,
            complete_ctx,
            prompt_ctx,
        );
        Ok(ReadOutcome::Eof)
    }
}
