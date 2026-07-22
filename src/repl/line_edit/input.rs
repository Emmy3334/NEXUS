//! Detect a real terminal on the *stdin handle*, not process fd 0 alone.
//!
//! Tests feed a [`std::io::Cursor`]; that must never enable raw-mode editing
//! even when the cargo process itself is attached to a TTY.

use std::io::{BufRead, Cursor, IsTerminal, StdinLock};

pub trait ReplInput: BufRead {
    fn is_terminal(&self) -> bool;
}

impl ReplInput for StdinLock<'_> {
    fn is_terminal(&self) -> bool {
        IsTerminal::is_terminal(self)
    }
}

impl<T: AsRef<[u8]>> ReplInput for Cursor<T> {
    fn is_terminal(&self) -> bool {
        false
    }
}

impl<I: ReplInput + ?Sized> ReplInput for &mut I {
    fn is_terminal(&self) -> bool {
        (**self).is_terminal()
    }
}
