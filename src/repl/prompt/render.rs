//! Assemble classic or Powerlevel10k-inspired primary prompts.

use super::color;
use super::context::PromptContext;
use super::git;
use super::segments;
use super::style::Style;
use super::PRIMARY_BARE;

#[must_use]
pub fn render(ctx: &PromptContext) -> String {
    match ctx.style {
        Style::Classic => classic(ctx),
        Style::Powerlevel10k => powerlevel(ctx),
    }
}

fn classic(_ctx: &PromptContext) -> String {
    match git::segment() {
        Some(seg) => format!("$> [{seg}] "),
        None => PRIMARY_BARE.to_owned(),
    }
}

fn powerlevel(ctx: &PromptContext) -> String {
    let mut out = String::new();
    for (i, el) in ctx.elements.iter().enumerate() {
        let part = segments::render(*el, ctx);
        if part.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        // Keep a space before prompt_char even if previous was empty—handled above.
        let _ = i;
        out.push_str(&part);
    }
    out.push(' ');
    out.push_str(color::reset());
    out
}
