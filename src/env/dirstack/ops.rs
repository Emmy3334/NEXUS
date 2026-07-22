//! Mutating operations on [`super::DirStack`].

use super::DirStack;

use std::path::PathBuf;

impl DirStack {
    pub fn push(&mut self, path: PathBuf) {
        self.entries.insert(0, path);
    }

    pub fn pop(&mut self) -> Option<PathBuf> {
        if self.entries.is_empty() {
            None
        } else {
            Some(self.entries.remove(0))
        }
    }

    /// Rotate so entry `n` becomes the top (`0`). Returns `false` if `n` is out of range.
    #[must_use]
    pub fn rotate_left(&mut self, n: usize) -> bool {
        if n == 0 {
            return true;
        }
        if n >= self.entries.len() {
            return false;
        }
        self.entries.rotate_left(n);
        true
    }

    pub fn remove_at(&mut self, n: usize) -> Option<PathBuf> {
        if n >= self.entries.len() {
            None
        } else {
            Some(self.entries.remove(n))
        }
    }

    /// Clear the stack and keep only `cwd` (tcsh `dirs -c`).
    pub fn reset_to(&mut self, cwd: PathBuf) {
        self.entries.clear();
        self.entries.push(cwd);
    }

    /// Update stack top to match cwd after `cd` (tcsh keeps dirstack[0] in sync).
    pub fn set_top(&mut self, path: PathBuf) {
        if self.entries.is_empty() {
            self.entries.push(path);
        } else {
            self.entries[0] = path;
        }
    }
}
