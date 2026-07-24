//! Prompt strings for primary and continuation lines.

mod color;
mod context;
mod git;
mod line;
mod render;
mod segments;
mod style;
mod width;
mod write;

pub use context::PromptContext;
pub(crate) use line::PromptLine;
pub use width::visible_columns;
pub(crate) use write::{write_continue, write_primary, write_primary_ctx};

use crate::env::ShellEnvironment;

/// Continuation prompt (multi-line / open quotes).
pub(super) const CONTINUE: &str = "? ";

/// Bare primary prompt when not inside a git work tree.
pub(super) const PRIMARY_BARE: &str = "$> ";

/// Build the interactive primary prompt (`$> ` or themed segments).
#[must_use]
pub fn format_primary() -> String {
    format_primary_with(&PromptContext::classic_default())
}

/// Build the primary prompt from an explicit context (theme + status).
#[must_use]
pub fn format_primary_with(ctx: &PromptContext) -> String {
    render::render(ctx)
}

/// Convenience: resolve context from the live shell environment.
#[must_use]
pub fn format_primary_env(env: &ShellEnvironment, last_status: u8) -> String {
    format_primary_with(&PromptContext::from_env(env, last_status))
}

/// True when `prompt` is a primary line (history recall enabled).
#[must_use]
pub(super) fn is_primary(prompt: &str) -> bool {
    prompt != CONTINUE
}
