//! Emacs / vi style resets.

use super::defaults::{seed_emacs, seed_vi};
use super::{EditorStyle, KeyBindings};

use std::collections::HashMap;

impl KeyBindings {
    #[must_use]
    pub fn new() -> Self {
        let mut primary = HashMap::new();
        let mut alternate = HashMap::new();
        seed_emacs(&mut primary, &mut alternate);
        Self {
            style: EditorStyle::Emacs,
            primary,
            alternate,
            alternate_active: false,
        }
    }

    /// `-d` — restore bindings for the current default editor style.
    pub fn reset_default(&mut self) {
        match self.style {
            EditorStyle::Emacs => self.reset_emacs(),
            EditorStyle::Vi => self.reset_vi(),
        }
    }

    /// `-e` — GNU Emacs-like bindings.
    pub fn reset_emacs(&mut self) {
        self.style = EditorStyle::Emacs;
        self.primary.clear();
        self.alternate.clear();
        seed_emacs(&mut self.primary, &mut self.alternate);
        self.alternate_active = false;
    }

    /// `-v` — vi-like bindings (insert + command maps).
    pub fn reset_vi(&mut self) {
        self.style = EditorStyle::Vi;
        self.primary.clear();
        self.alternate.clear();
        seed_vi(&mut self.primary, &mut self.alternate);
        self.alternate_active = false;
    }
}
