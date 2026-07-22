//! Read one key event from stdin in raw mode.

use super::term::nix_err;
use crate::repl::line_edit::bindings::Action;
use nix::unistd::read;

use std::io;
use std::os::fd::AsRawFd;

#[derive(Debug)]
pub(super) enum Event {
    Action(Action),
    Insert(char),
    Raw(Vec<u8>),
}

pub(super) fn read_event() -> io::Result<Event> {
    let mut buf = [0u8; 8];
    let n = read(io::stdin().as_raw_fd(), &mut buf).map_err(nix_err)?;
    if n == 0 {
        return Ok(Event::Action(Action::Eof));
    }
    let slice = &buf[..n];
    if let Some(action) = map_simple(slice) {
        return Ok(Event::Action(action));
    }
    if slice[0] == 0x1b {
        return Ok(Event::Raw(slice.to_vec()));
    }
    if slice[0] < 0x20 {
        return Ok(Event::Raw(slice.to_vec()));
    }
    let text = std::str::from_utf8(slice).unwrap_or("");
    if let Some(ch) = text.chars().next() {
        return Ok(Event::Insert(ch));
    }
    Ok(Event::Raw(slice.to_vec()))
}

fn map_simple(keys: &[u8]) -> Option<Action> {
    match keys {
        [b'\r'] | [b'\n'] => Some(Action::Accept),
        [0x7f] | [0x08] => Some(Action::Backspace),
        [b'\t'] => Some(Action::Complete),
        [0x03] => Some(Action::Interrupt),
        [0x04] => Some(Action::Eof),
        _ => None,
    }
}
