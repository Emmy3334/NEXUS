//! Event handling including bracketed-paste mode.

mod dispatch;

use super::actions::Loop;
use super::buffer::EditBuffer;
use super::draw;
use super::event::Event;
use super::isearch_mode;
use super::keys::read_event;
use crate::history::History;
use crate::keybind::KeyBindings;
use crate::repl::line_edit::isearch::HistoryISearch;
use crate::repl::line_edit::recall::HistoryRecall;
use crate::repl::prompt::PromptLine;

use std::collections::VecDeque;
use std::io::{self, Write};

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_event<'a>(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    bindings: &mut KeyBindings,
    prompt: &mut PromptLine,
    nav: &mut HistoryRecall<'_>,
    queue: &mut VecDeque<u8>,
    pasting: &mut bool,
    history: &'a History,
    isearch: &mut Option<HistoryISearch<'a>>,
    var_names: &[String],
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
        Event::Action(action) if !*pasting => dispatch::action(
            stdout, edit, bindings, prompt, nav, history, isearch, action, var_names,
        ),
        Event::Action(_) => Ok(Loop::Continue),
        Event::InsertRun(text) if *pasting => insert_text(stdout, edit, prompt.as_str(), &text),
        Event::InsertRun(text) => match isearch.as_mut() {
            Some(search) => isearch_mode::on_insert(stdout, edit, search, &text),
            None if !bindings.alternate_active() => {
                insert_text(stdout, edit, prompt.as_str(), &text)
            }
            None => dispatch::insert_bound(
                stdout,
                edit,
                bindings,
                prompt.as_str(),
                nav,
                &text,
                var_names,
            ),
        },
        Event::Raw(bytes) if *pasting => insert_raw_paste(stdout, edit, prompt.as_str(), &bytes),
        Event::Raw(bytes) if isearch.is_some() => dispatch::isearch_raw(
            stdout, edit, bindings, prompt, nav, history, isearch, &bytes, var_names,
        ),
        Event::Raw(bytes) => dispatch::raw(
            stdout,
            edit,
            bindings,
            prompt.as_str(),
            nav,
            &bytes,
            var_names,
        ),
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
