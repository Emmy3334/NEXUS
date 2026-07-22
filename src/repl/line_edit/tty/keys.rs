//! Read one key event from stdin in raw mode (queue-backed for paste).

use super::escape;
use super::event::Event;
use super::queue::{self, fill};
use crate::repl::line_edit::bindings::Action;

use std::collections::VecDeque;
use std::io;

pub(super) fn read_event(q: &mut VecDeque<u8>) -> io::Result<Event> {
    if !fill(q)? {
        return Ok(Event::Action(Action::Eof));
    }
    let Some(&b0) = q.front() else {
        return Ok(Event::Action(Action::Eof));
    };
    if b0 == 0x1b {
        return escape::decode_escape(q);
    }
    if let Some(action) = map_control(b0) {
        q.pop_front();
        return Ok(Event::Action(action));
    }
    if b0 < 0x20 {
        q.pop_front();
        return Ok(Event::Raw(vec![b0]));
    }
    Ok(Event::InsertRun(take_printable_run(q)?))
}

fn map_control(byte: u8) -> Option<Action> {
    match byte {
        b'\r' | b'\n' => Some(Action::Accept),
        0x7f | 0x08 => Some(Action::Backspace),
        b'\t' => Some(Action::Complete),
        0x03 => Some(Action::Interrupt),
        0x04 => Some(Action::Eof),
        _ => None,
    }
}

fn take_printable_run(q: &mut VecDeque<u8>) -> io::Result<String> {
    let mut out = String::new();
    while let Some(&b) = q.front() {
        if b < 0x20 || b == 0x7f {
            break;
        }
        let need = utf8_width(b);
        while q.len() < need && queue::fill_more(q)? {}
        if q.len() < need {
            break;
        }
        let mut buf = [0u8; 4];
        for slot in buf.iter_mut().take(need) {
            let Some(byte) = q.pop_front() else {
                return Ok(out);
            };
            *slot = byte;
        }
        if let Ok(text) = std::str::from_utf8(&buf[..need]) {
            out.push_str(text);
        }
    }
    Ok(out)
}

fn utf8_width(first: u8) -> usize {
    match first {
        0x00..0x80 => 1,
        0x80..0xe0 => 2,
        0xe0..0xf0 => 3,
        _ => 4,
    }
}
