//! Interactive editor key bindings (tcsh-style `bindkey`).

mod defaults;
mod display;
mod methods;
mod mode;
mod parse;
mod style;

pub use display::{action_name, binding_label, format_key};
pub use parse::{parse_action, parse_key, parse_key_b};

use std::collections::HashMap;

/// Action performed when a key sequence is pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Accept,
    Backspace,
    Delete,
    MoveLeft,
    MoveRight,
    MoveHome,
    MoveEnd,
    MoveWordLeft,
    MoveWordRight,
    KillWordForward,
    KillWordBackward,
    KillToEol,
    KillLine,
    Yank,
    TransposeWords,
    HistoryUp,
    HistoryDown,
    /// Reverse incremental history search (Ctrl-R).
    HistoryISearch,
    Complete,
    Interrupt,
    Eof,
    /// Enter vi command-mode map (`-a` / ESC).
    ViCmdMode,
    /// Return to vi insert map.
    ViInsertMode,
}

/// What a key is bound to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Binding {
    Action(Action),
    /// `bindkey -c` — run as a shell command line.
    Command(String),
    /// `bindkey -s` — insert this text as typed.
    Literal(String),
}

/// Which default set is active (`-e` / `-v` / `-d`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorStyle {
    Emacs,
    Vi,
}

/// Mutable primary + alternate key maps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBindings {
    style: EditorStyle,
    primary: HashMap<Vec<u8>, Binding>,
    alternate: HashMap<Vec<u8>, Binding>,
    /// When true, lookups use the alternate (`-a` / vi cmd) map.
    alternate_active: bool,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::new()
    }
}
