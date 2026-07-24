//! Prompt character (success / error coloring).

use super::super::color::{self, BLUE, RED};
use super::super::context::PromptContext;
use super::icons::Glyphs;

#[must_use]
pub fn render(ctx: &PromptContext) -> String {
    let g = Glyphs::for_mode(ctx.icons);
    let (ch, code) = if ctx.last_status == 0 {
        (g.prompt_ok, BLUE)
    } else {
        (g.prompt_err, RED)
    };
    color::fg(code, ch)
}
