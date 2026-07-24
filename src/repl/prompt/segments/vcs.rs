//! Git / VCS segment with branch icon and dirty coloring.

use super::super::color::{self, MAGENTA, YELLOW};
use super::super::context::PromptContext;
use super::super::git;
use super::icons::Glyphs;

#[must_use]
pub fn render(ctx: &PromptContext) -> String {
    let Some(info) = git::info() else {
        return String::new();
    };
    let g = Glyphs::for_mode(ctx.icons);
    let mut body = String::new();
    push_icon(&mut body, g.git);
    push_icon(&mut body, g.branch);
    body.push_str(&info.branch);
    if info.dirty {
        body.push('*');
    }
    let code = if info.dirty { YELLOW } else { MAGENTA };
    color::fg(code, &body)
}

fn push_icon(out: &mut String, icon: &str) {
    if icon.is_empty() {
        return;
    }
    out.push_str(icon);
    out.push(' ');
}
