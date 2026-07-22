//! Decode CSI / SS3 escape sequences from the input queue.

use super::event::Event;
use super::queue;

use std::collections::VecDeque;
use std::io;

pub(super) fn decode_escape(q: &mut VecDeque<u8>) -> io::Result<Event> {
    while q.len() < 3 && queue::fill_more(q)? {}
    let seq = peek_escape(q);
    if seq == b"\x1b[200~" {
        drain(q, seq.len());
        return Ok(Event::PasteStart);
    }
    if seq == b"\x1b[201~" {
        drain(q, seq.len());
        return Ok(Event::PasteEnd);
    }
    if is_complete_escape(&seq) {
        drain(q, seq.len());
        return Ok(Event::Raw(seq));
    }
    q.pop_front();
    Ok(Event::Raw(vec![0x1b]))
}

fn drain(q: &mut VecDeque<u8>, n: usize) {
    for _ in 0..n {
        q.pop_front();
    }
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

fn is_complete_escape(seq: &[u8]) -> bool {
    matches!(
        seq,
        b"\x1b[D" | b"\x1b[C" | b"\x1b[A" | b"\x1b[B" | b"\x1b[3~" | [0x1b, b'O', _]
    )
}
