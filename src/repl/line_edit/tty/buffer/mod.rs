//! Core UTF-8 edit buffer storage.

mod home;
mod kill;
mod nav;
mod transpose;
mod vline;
mod word;

use std::collections::VecDeque;

pub(super) use home::{move_end, move_home};
pub(super) use kill::{kill_line, kill_to_eol, kill_word_backward, kill_word_forward, yank};
pub(super) use nav::{backspace, delete, move_left, move_right};
pub(super) use transpose::transpose_words;
pub use vline::{after_line_down, after_line_up};
pub(super) use vline::{move_line_down, move_line_up};
pub(super) use word::{move_word_left, move_word_right};

#[derive(Debug, Default)]
pub(super) struct EditBuffer {
    pub(super) text: String,
    pub(super) cursor: usize,
    pub(super) kill_ring: VecDeque<String>,
    /// Ambiguous Tab cycle; cleared on any non-Complete edit.
    pub(super) complete_cycle: Option<super::super::complete::CompleteCycle>,
}

impl EditBuffer {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn as_str(&self) -> &str {
        &self.text
    }

    pub(super) fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub(super) fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.complete_cycle = None;
    }

    pub(super) fn insert(&mut self, ch: char) {
        self.complete_cycle = None;
        self.text.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
    }

    pub(super) fn push_char(&mut self, ch: char) {
        self.complete_cycle = None;
        self.text.push(ch);
        self.cursor = self.text.len();
    }
}
