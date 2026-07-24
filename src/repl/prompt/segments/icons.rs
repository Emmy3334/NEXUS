//! Nerd Font vs ASCII glyphs for themed prompts.

use super::super::style::IconMode;

pub struct Glyphs {
    pub os_macos: &'static str,
    pub os_linux: &'static str,
    pub os_other: &'static str,
    pub folder: &'static str,
    pub git: &'static str,
    pub branch: &'static str,
    pub prompt_ok: &'static str,
    pub prompt_err: &'static str,
}

impl Glyphs {
    #[must_use]
    pub fn for_mode(mode: IconMode) -> Self {
        match mode {
            IconMode::NerdFont => Self::nerd(),
            IconMode::Ascii => Self::ascii(),
        }
    }

    fn nerd() -> Self {
        Self {
            os_macos: "\u{f179}", // nf-fa-apple
            os_linux: "\u{f17c}",
            os_other: "\u{f109}",
            folder: "\u{f07c}",
            git: "\u{f113}",
            branch: "\u{f126}",
            prompt_ok: "❯",
            prompt_err: "❯",
        }
    }

    fn ascii() -> Self {
        Self {
            os_macos: "",
            os_linux: "",
            os_other: "",
            folder: "",
            git: "git",
            branch: "",
            prompt_ok: ">",
            prompt_err: ">",
        }
    }
}
