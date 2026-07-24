//! Current-directory segment with home and unique-prefix shortening.

mod prefix;
mod shorten;
mod tilde;

use super::super::color::{self, GREEN, RED};
use super::super::context::PromptContext;
use super::icons::Glyphs;

#[must_use]
pub fn render(ctx: &PromptContext) -> String {
    let display = shorten::display_path(&ctx.cwd, &ctx.home);
    let g = Glyphs::for_mode(ctx.icons);
    // p10k lean_8colors: folder icon red (1), path green (2).
    let mut out = String::new();
    if !g.folder.is_empty() {
        out.push_str(&color::fg(RED, g.folder));
        out.push(' ');
    }
    out.push_str(&color::fg(GREEN, &display));
    out
}
