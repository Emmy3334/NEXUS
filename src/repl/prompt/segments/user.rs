//! Username segment (yellow, matching p10k lean_8colors).

use super::super::color::{self, YELLOW};
use super::super::context::PromptContext;

#[must_use]
pub fn render(ctx: &PromptContext) -> String {
    if ctx.user.is_empty() {
        return String::new();
    }
    color::fg(YELLOW, &ctx.user)
}
