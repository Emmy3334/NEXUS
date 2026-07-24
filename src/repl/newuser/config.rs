//! Settings collected by the new-user wizard.

/// User choices that become lines in `.nexusrc`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub histsize: u32,
    pub prompt_style: PromptStyle,
    pub keymap: Keymap,
    pub oh_my_nexus: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            histsize: 10_000,
            prompt_style: PromptStyle::Classic,
            keymap: Keymap::Emacs,
            oh_my_nexus: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptStyle {
    Classic,
    Powerlevel10k,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keymap {
    Emacs,
    Vi,
}
