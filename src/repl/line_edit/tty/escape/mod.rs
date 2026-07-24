//! Decode CSI / SS3 / Meta escape sequences from the input queue.

mod classify;

use super::event::Event;
use super::queue;
use classify::{is_csi_final, is_finished};

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
    if is_finished(&seq) {
        drain(q, seq.len());
        return Ok(Event::Raw(seq));
    }
    // Incomplete CSI/SS3: try one more fill, then drain the prefix entirely so
    // leftovers like `}` / `[1;2A` never land in the line buffer.
    if seq.len() > 1 {
        while q.len() < seq.len() + 1 && queue::fill_more(q)? {}
        let seq = peek_escape(q);
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
        if seq.get(1) == Some(&b'O') && seq.len() >= 3 {
            break;
        }
        if i >= 2 && is_csi_final(b) {
            break;
        }
        if seq.len() >= 32 {
            break;
        }
    }
    seq
}
