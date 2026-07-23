//! Core UTF-8 edit buffer storage.

mod kill;
mod nav;
mod transpose;
mod word;

use std::collections::VecDeque;

pub(super) use kill::{kill_line, kill_to_eol, kill_word_backward, kill_word_forward, yank};
pub(super) use nav::{backspace, delete, move_left, move_right};
pub(super) use transpose::transpose_words;
pub(super) use word::{move_word_left, move_word_right};

#[derive(Debug, Default)]
pub(super) struct EditBuffer {
    pub(super) text: String,
    pub(super) cursor: usize,
    pub(super) kill_ring: VecDeque<String>,
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
    }

    pub(super) fn insert(&mut self, ch: char) {
        self.text.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
    }

    pub(super) fn push_char(&mut self, ch: char) {
        self.text.push(ch);
        self.cursor = self.text.len();
    }
}
