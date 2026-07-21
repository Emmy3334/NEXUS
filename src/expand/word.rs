//! Result of quote / `$` expansion, with per-character glob activity flags.

/// Expanded text plus parallel glob-activity flags.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExpandedWord {
    pub(super) text: String,
    /// Parallel to `text.chars()`: whether that character is an active glob meta.
    pub(super) glob_meta: Vec<bool>,
}

impl ExpandedWord {
    /// Expanded text (quotes already stripped).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Consume into an owned string (drops glob flags).
    #[must_use]
    pub fn into_string(self) -> String {
        self.text
    }

    /// Whether any active glob metacharacter is present.
    #[must_use]
    pub fn has_active_glob(&self) -> bool {
        self.glob_meta.iter().any(|&m| m)
    }

    /// Borrow the parallel glob-activity flags (one per Unicode scalar in `text`).
    #[must_use]
    pub fn glob_meta(&self) -> &[bool] {
        &self.glob_meta
    }

    pub(super) fn clear(&mut self) {
        self.text.clear();
        self.glob_meta.clear();
    }

    pub(super) fn push_literal(&mut self, c: char) {
        self.text.push(c);
        self.glob_meta.push(false);
    }

    pub(super) fn push_glob_meta(&mut self, c: char) {
        debug_assert!(matches!(c, '*' | '?' | '['));
        self.text.push(c);
        self.glob_meta.push(true);
    }
}
