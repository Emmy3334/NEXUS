//! Owned primary / continuation prompt for the TTY editor.

use super::{format_primary_with, PromptContext, CONTINUE};

/// Prompt text that can switch between primary and continuation forms.
pub(crate) enum PromptLine {
    Primary(String),
    Continue,
}

impl PromptLine {
    #[must_use]
    pub(crate) fn from_ctx(ctx: &PromptContext) -> Self {
        Self::Primary(format_primary_with(ctx))
    }

    #[must_use]
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Primary(text) => text.as_str(),
            Self::Continue => CONTINUE,
        }
    }

    pub(crate) fn set_from_ctx(&mut self, ctx: &PromptContext) {
        *self = Self::from_ctx(ctx);
    }

    pub(crate) fn set_continue(&mut self) {
        *self = Self::Continue;
    }
}
