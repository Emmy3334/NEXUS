//! Prompt style flags resolved from the shell environment.

/// How the primary prompt is assembled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// Legacy `$> ` / `$> [branch*] `.
    Classic,
    /// Powerlevel10k-inspired left segments.
    Powerlevel10k,
}

/// Icon glyph set for themed prompts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconMode {
    NerdFont,
    Ascii,
}

/// Named left-prompt segments (p10k-inspired).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Element {
    OsIcon,
    User,
    Dir,
    Vcs,
    PromptChar,
}

impl Style {
    #[must_use]
    pub fn parse(raw: Option<&str>) -> Self {
        match raw.map(str::trim) {
            Some("powerlevel10k" | "p10k") => Self::Powerlevel10k,
            _ => Self::Classic,
        }
    }
}

impl IconMode {
    #[must_use]
    pub fn parse(raw: Option<&str>) -> Self {
        match raw.map(str::trim) {
            Some("ascii" | "none") => Self::Ascii,
            _ => Self::NerdFont,
        }
    }
}
