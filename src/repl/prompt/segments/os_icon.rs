//! OS identifier segment.

use super::super::context::PromptContext;
use super::icons::Glyphs;

#[must_use]
pub fn render(ctx: &PromptContext) -> String {
    let g = Glyphs::for_mode(ctx.icons);
    let icon = if cfg!(target_os = "macos") {
        g.os_macos
    } else if cfg!(target_os = "linux") {
        g.os_linux
    } else {
        g.os_other
    };
    icon.to_owned()
}
