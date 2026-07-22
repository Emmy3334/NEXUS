//! Event handling including bracketed-paste mode.

use super::actions::{self, Loop};
use super::buffer::EditBuffer;
use super::draw;
use super::event::Event;
use super::keys::read_event;
use crate::keybind::{Binding, KeyBindings};
use crate::repl::line_edit::recall::HistoryRecall;
use crate::repl::prompt::PromptLine;

use std::collections::VecDeque;
use std::io::{self, Write};

pub(super) fn handle_event(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    bindings: &mut KeyBindings,
    prompt: &mut PromptLine,
    nav: &mut HistoryRecall<'_>,
    queue: &mut VecDeque<u8>,
    pasting: &mut bool,
) -> io::Result<Loop> {
    match read_event(queue)? {
        Event::PasteStart => {
            *pasting = true;
            Ok(Loop::Continue)
        }
        Event::PasteEnd => {
            *pasting = false;
            Ok(Loop::Continue)
        }
        Event::Action(action) if !*pasting => {
            actions::apply(stdout, edit, bindings, action, prompt.as_str(), nav)
        }
        Event::Action(_) => Ok(Loop::Continue),
        Event::InsertRun(text) if *pasting || !bindings.alternate_active() => {
            insert_text(stdout, edit, prompt.as_str(), &text)
        }
        Event::InsertRun(text) => insert_bound(stdout, edit, bindings, prompt.as_str(), nav, &text),
        Event::Raw(bytes) if *pasting => insert_raw_paste(stdout, edit, prompt.as_str(), &bytes),
        Event::Raw(bytes) => dispatch_raw(stdout, edit, bindings, prompt.as_str(), nav, &bytes),
    }
}

fn insert_text(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    prompt: &str,
    text: &str,
) -> io::Result<Loop> {
    for ch in text.chars() {
        edit.insert(ch);
    }
    draw::redraw(stdout, prompt, edit)?;
    Ok(Loop::Continue)
}

fn insert_raw_paste(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    prompt: &str,
    bytes: &[u8],
) -> io::Result<Loop> {
    for &b in bytes {
        let ch = match b {
            b'\r' | b'\n' => '\n',
            b if b >= 0x20 && b != 0x7f => b as char,
            _ => continue,
        };
        edit.insert(ch);
    }
    draw::redraw(stdout, prompt, edit)?;
    Ok(Loop::Continue)
}

fn dispatch_raw(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    bindings: &mut KeyBindings,
    prompt: &str,
    nav: &mut HistoryRecall<'_>,
    bytes: &[u8],
) -> io::Result<Loop> {
    match bindings.lookup(bytes).cloned() {
        Some(binding) => apply_binding(stdout, edit, bindings, prompt, nav, binding),
        None => Ok(Loop::Continue),
    }
}

fn insert_bound(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    bindings: &mut KeyBindings,
    prompt: &str,
    nav: &mut HistoryRecall<'_>,
    text: &str,
) -> io::Result<Loop> {
    for ch in text.chars() {
        let mut buf = [0u8; 4];
        let encoded = ch.encode_utf8(&mut buf);
        if let Some(binding) = bindings.lookup(encoded.as_bytes()).cloned() {
            let result = apply_binding(stdout, edit, bindings, prompt, nav, binding)?;
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
) -> io::Result<Loop> {
    match binding {
        Binding::Action(action) => actions::apply(stdout, edit, bindings, action, prompt, nav),
        Binding::Command(cmd) => {
            edit.clear();
            for ch in cmd.chars() {
                edit.insert(ch);
            }
            Ok(Loop::Accept)
        }
        Binding::Literal(text) => insert_text(stdout, edit, prompt, &text),
    }
}
