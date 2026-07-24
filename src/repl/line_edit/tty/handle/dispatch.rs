//! Binding dispatch helpers for the TTY event loop.

use super::super::actions::{self, Loop};
use super::super::buffer::EditBuffer;
use super::super::complete_menu;
use super::super::draw;
use super::super::isearch_mode;
use crate::history::History;
use crate::keybind::{Action, Binding, KeyBindings};
use crate::repl::line_edit::complete::CompleteCtx;
use crate::repl::line_edit::isearch::HistoryISearch;
use crate::repl::line_edit::recall::HistoryRecall;
use crate::repl::prompt::PromptLine;

use std::io::{self, Write};

#[allow(clippy::too_many_arguments)]
pub(super) fn action<'a>(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    bindings: &mut KeyBindings,
    prompt: &mut PromptLine,
    nav: &mut HistoryRecall<'_>,
    history: &'a History,
    isearch: &mut Option<HistoryISearch<'a>>,
    action: Action,
    complete_ctx: &CompleteCtx<'_>,
) -> io::Result<Loop> {
    if let Some(result) = isearch_mode::on_action(stdout, edit, isearch, action, prompt.as_str())? {
        return Ok(result);
    }
    if action == Action::HistoryISearch {
        return isearch_mode::begin(stdout, edit, history, isearch);
    }
    actions::apply(
        stdout,
        edit,
        bindings,
        action,
        prompt.as_str(),
        nav,
        complete_ctx,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn isearch_raw<'a>(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    bindings: &mut KeyBindings,
    prompt: &mut PromptLine,
    nav: &mut HistoryRecall<'_>,
    history: &'a History,
    isearch: &mut Option<HistoryISearch<'a>>,
    bytes: &[u8],
    complete_ctx: &CompleteCtx<'_>,
) -> io::Result<Loop> {
    match bindings.lookup(bytes).cloned() {
        Some(Binding::Action(act)) => action(
            stdout,
            edit,
            bindings,
            prompt,
            nav,
            history,
            isearch,
            act,
            complete_ctx,
        ),
        Some(binding) => {
            isearch_mode::abort(edit, isearch);
            apply_binding(
                stdout,
                edit,
                bindings,
                prompt.as_str(),
                nav,
                binding,
                complete_ctx,
            )
        }
        None if bytes == [0x1b] => {
            isearch_mode::abort(edit, isearch);
            isearch_mode::redraw_primary(stdout, edit, prompt.as_str())?;
            Ok(Loop::Continue)
        }
        None => Ok(Loop::Continue),
    }
}

pub(super) fn raw(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    bindings: &mut KeyBindings,
    prompt: &str,
    nav: &mut HistoryRecall<'_>,
    bytes: &[u8],
    complete_ctx: &CompleteCtx<'_>,
) -> io::Result<Loop> {
    match bindings.lookup(bytes).cloned() {
        Some(binding) => apply_binding(stdout, edit, bindings, prompt, nav, binding, complete_ctx),
        None if bytes == [0x1b]
            && edit
                .complete_cycle
                .as_ref()
                .is_some_and(|c| c.is_active(&edit.text, edit.cursor)) =>
        {
            complete_menu::dismiss_menu(stdout, edit)?;
            draw::redraw(stdout, prompt, edit)?;
            Ok(Loop::Continue)
        }
        None if bytes.len() == 2 && bytes[0] == 0x1b => {
            if matches!(
                bindings.lookup(&[0x1b]),
                Some(Binding::Action(Action::ViCmdMode))
            ) {
                bindings.enter_command_map();
                return raw(
                    stdout,
                    edit,
                    bindings,
                    prompt,
                    nav,
                    &bytes[1..],
                    complete_ctx,
                );
            }
            Ok(Loop::Continue)
        }
        None => Ok(Loop::Continue),
    }
}

pub(super) fn insert_bound(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    bindings: &mut KeyBindings,
    prompt: &str,
    nav: &mut HistoryRecall<'_>,
    text: &str,
    complete_ctx: &CompleteCtx<'_>,
) -> io::Result<Loop> {
    for ch in text.chars() {
        let mut buf = [0u8; 4];
        let encoded = ch.encode_utf8(&mut buf);
        if let Some(binding) = bindings.lookup(encoded.as_bytes()).cloned() {
            let result = apply_binding(stdout, edit, bindings, prompt, nav, binding, complete_ctx)?;
            if !matches!(result, Loop::Continue) {
                return Ok(result);
            }
        }
    }
    Ok(Loop::Continue)
}

fn apply_binding(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    bindings: &mut KeyBindings,
    prompt: &str,
    nav: &mut HistoryRecall<'_>,
    binding: Binding,
    complete_ctx: &CompleteCtx<'_>,
) -> io::Result<Loop> {
    match binding {
        Binding::Action(action) => {
            actions::apply(stdout, edit, bindings, action, prompt, nav, complete_ctx)
        }
        Binding::Command(cmd) => {
            complete_menu::dismiss_menu(stdout, edit)?;
            edit.clear();
            for ch in cmd.chars() {
                edit.insert(ch);
            }
            Ok(Loop::Accept)
        }
        Binding::Literal(text) => {
            complete_menu::dismiss_menu(stdout, edit)?;
            for ch in text.chars() {
                edit.insert(ch);
            }
            draw::redraw(stdout, prompt, edit)?;
            Ok(Loop::Continue)
        }
    }
}
