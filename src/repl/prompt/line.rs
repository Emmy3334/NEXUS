//! Owned primary / continuation prompt for the TTY editor.

use super::{format_primary, CONTINUE};

/// Prompt text that can switch between primary and continuation forms.
pub(crate) enum PromptLine {
    Primary(String),
    Continue,
}

impl PromptLine {
    #[must_use]
    pub(crate) fn primary() -> Self {
        Self::Primary(format_primary())
    }

    #[must_use]
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Primary(text) => text.as_str(),
            Self::Continue => CONTINUE,
        }
    }

    pub(crate) fn set_primary(&mut self) {
        *self = Self::primary();
    }

    pub(crate) fn set_continue(&mut self) {
        *self = Self::Continue;
    }
}
