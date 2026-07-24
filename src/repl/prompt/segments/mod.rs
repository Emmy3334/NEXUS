//! Left-prompt segment renderers (Powerlevel10k-inspired).

mod dir;
mod icons;
mod os_icon;
mod prompt_char;
mod user;
mod vcs;

use super::context::PromptContext;
use super::style::Element;

/// Render one configured segment; empty string means “skip”.
#[must_use]
pub fn render(element: Element, ctx: &PromptContext) -> String {
    match element {
        Element::OsIcon => os_icon::render(ctx),
        Element::User => user::render(ctx),
        Element::Dir => dir::render(ctx),
        Element::Vcs => vcs::render(ctx),
        Element::PromptChar => prompt_char::render(ctx),
    }
}
