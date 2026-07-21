//! Push helpers for [`ExpandedWord`] string fragments.

use super::ExpandedWord;

impl ExpandedWord {
    pub(super) fn push_str_literal(&mut self, s: &str) {
        for c in s.chars() {
            self.push_literal(c);
        }
    }

    /// Unquoted expansion: `*`, `?`, `[` in the value become active metas.
    pub(super) fn push_str_globable(&mut self, s: &str) {
        for c in s.chars() {
            match c {
                '*' | '?' | '[' => self.push_glob_meta(c),
                _ => self.push_literal(c),
            }
        }
    }
}
