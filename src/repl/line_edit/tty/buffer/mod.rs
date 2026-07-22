//! Core UTF-8 edit buffer storage.

mod nav;

pub(super) use nav::{backspace, delete, move_left, move_right};

#[derive(Debug, Default)]
pub(super) struct EditBuffer {
    pub(super) text: String,
    pub(super) cursor: usize,
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
