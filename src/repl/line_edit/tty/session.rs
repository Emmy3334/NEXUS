//! Raw-mode read loop and accept / interrupt handling.

use super::super::ReadOutcome;
use super::actions::Loop;
use super::buffer::EditBuffer;
use super::draw;
use super::handle;
use super::queue;
use crate::history::History;
use crate::keybind::KeyBindings;
use crate::repl::line_edit::complete::CompleteCtx;
use crate::repl::line_edit::isearch::HistoryISearch;
use crate::repl::line_edit::probe::quotes_closed;
use crate::repl::line_edit::recall::HistoryRecall;
use crate::repl::prompt::{PromptContext, PromptLine};

use std::collections::VecDeque;
use std::io::{self, Write};

#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    stdout: &mut impl Write,
    out: &mut String,
    history: &History,
    bindings: &mut KeyBindings,
    queue: &mut VecDeque<u8>,
    complete_ctx: &CompleteCtx<'_>,
    prompt_ctx: &PromptContext,
) -> io::Result<ReadOutcome> {
    let mut edit = EditBuffer::new();
    let mut nav = HistoryRecall::new(history);
    let mut prompt = PromptLine::from_ctx(prompt_ctx);
    let (mut pasting, mut isearch) = (false, None::<HistoryISearch<'_>>);
    bindings.enter_insert_map();
    draw::redraw(stdout, prompt.as_str(), &edit)?;
    loop {
        if let Some(done) = poll(
            stdout,
            &mut edit,
            bindings,
            &mut prompt,
            prompt_ctx,
            &mut nav,
            queue,
            &mut pasting,
            history,
            &mut isearch,
            out,
            complete_ctx,
        )? {
            return Ok(done);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn poll<'a>(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    bindings: &mut KeyBindings,
    prompt: &mut PromptLine,
    prompt_ctx: &PromptContext,
    nav: &mut HistoryRecall<'a>,
    queue: &mut VecDeque<u8>,
    pasting: &mut bool,
    history: &'a History,
    isearch: &mut Option<HistoryISearch<'a>>,
    out: &mut String,
    complete_ctx: &CompleteCtx<'_>,
) -> io::Result<Option<ReadOutcome>> {
    let step = handle::handle_event(
        stdout,
        edit,
        bindings,
        prompt,
        nav,
        queue,
        pasting,
        history,
        isearch,
        complete_ctx,
    )?;
    apply_step(
        step, stdout, edit, prompt, prompt_ctx, out, queue, nav, history, isearch,
    )
}

#[allow(clippy::too_many_arguments)]
fn apply_step<'a>(
    step: Loop,
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    prompt: &mut PromptLine,
    prompt_ctx: &PromptContext,
    out: &mut String,
    queue: &mut VecDeque<u8>,
    nav: &mut HistoryRecall<'a>,
    history: &'a History,
    isearch: &mut Option<HistoryISearch<'a>>,
) -> io::Result<Option<ReadOutcome>> {
    match step {
        Loop::Continue => Ok(None),
        Loop::Accept => {
            *isearch = None;
            if finish_accept(stdout, edit, prompt, out, queue)? {
                return Ok(Some(ReadOutcome::Line));
            }
            *nav = HistoryRecall::new(history);
            Ok(None)
        }
        Loop::Eof if edit.is_empty() && isearch.is_none() => {
            out.clear();
            Ok(Some(ReadOutcome::Eof))
        }
        Loop::Eof => Ok(None),
        Loop::Interrupt => {
            *isearch = None;
            on_interrupt(stdout, edit, nav, history, prompt, prompt_ctx)?;
            Ok(None)
        }
    }
}

fn on_interrupt<'a>(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    nav: &mut HistoryRecall<'a>,
    history: &'a History,
    prompt: &mut PromptLine,
    prompt_ctx: &PromptContext,
) -> io::Result<()> {
    edit.clear();
    *nav = HistoryRecall::new(history);
    writeln!(stdout, "^C")?;
    prompt.set_from_ctx(prompt_ctx);
    draw::redraw(stdout, prompt.as_str(), edit)
}

fn finish_accept(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    prompt: &mut PromptLine,
    out: &mut String,
    queue: &mut VecDeque<u8>,
) -> io::Result<bool> {
    writeln!(stdout)?;
    if !quotes_closed(edit.as_str()) {
        edit.push_char('\n');
        prompt.set_continue();
        write!(stdout, "{}", prompt.as_str())?;
        stdout.flush()?;
        return Ok(false);
    }
    let text = std::mem::take(&mut edit.text);
    edit.cursor = 0;
    match text.split_once('\n') {
        Some((first, rest)) => {
            *out = first.to_owned();
            queue::push_front_lines(queue, rest);
        }
        None => *out = text,
    }
    Ok(true)
}
