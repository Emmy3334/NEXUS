//! Decode CSI / SS3 escape sequences from the input queue.

use super::event::Event;
use super::queue;
use crate::repl::line_edit::bindings::Action;

use std::collections::VecDeque;
use std::io;

pub(super) fn decode_escape(q: &mut VecDeque<u8>) -> io::Result<Event> {
    while q.len() < 3 && queue::fill_more(q)? {}
    let seq = peek_escape(q);
    if let Some(action) = map_escape(&seq) {
        for _ in 0..seq.len() {
            q.pop_front();
        }
        return Ok(Event::Action(action));
    }
    q.pop_front();
    Ok(Event::Raw(vec![0x1b]))
}

fn peek_escape(q: &VecDeque<u8>) -> Vec<u8> {
    let mut seq = Vec::new();
    for (i, &b) in q.iter().enumerate() {
        seq.push(b);
        if i == 0 {
            continue;
        }
        if i == 1 && b != b'[' && b != b'O' {
            break;
        }
        if i >= 2 && (b.is_ascii_alphabetic() || b == b'~') {
            break;
        }
        if seq.len() >= 6 {
            break;
        }
    }
    seq
}

fn map_escape(seq: &[u8]) -> Option<Action> {
    match seq {
        b"\x1b[D" => Some(Action::MoveLeft),
        b"\x1b[C" => Some(Action::MoveRight),
        b"\x1b[A" => Some(Action::HistoryUp),
        b"\x1b[B" => Some(Action::HistoryDown),
        b"\x1b[3~" => Some(Action::Delete),
        _ => None,
    }
}
