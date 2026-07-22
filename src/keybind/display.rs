//! Display helpers for keys and bindings.

use super::{Action, Binding};

/// tcsh-ish editor command name for `action`.
#[must_use]
pub fn action_name(action: Action) -> &'static str {
    match action {
        Action::Accept => "accept-line",
        Action::Backspace => "backward-delete-char",
        Action::Delete => "delete-char",
        Action::MoveLeft => "backward-char",
        Action::MoveRight => "forward-char",
        Action::HistoryUp => "up-history",
        Action::HistoryDown => "down-history",
        Action::Complete => "complete-word",
        Action::Interrupt => "tty-sigintr",
        Action::Eof => "end-of-file",
        Action::ViCmdMode => "vi-cmd-mode",
        Action::ViInsertMode => "vi-insert-mode",
    }
}

/// Label printed by `bindkey` for a binding.
#[must_use]
pub fn binding_label(binding: &Binding) -> String {
    match binding {
        Binding::Action(action) => action_name(*action).to_owned(),
        Binding::Command(cmd) => format!("run-command:{cmd}"),
        Binding::Literal(text) => format!("insert-string:{text}"),
    }
}

/// Render `keys` for `bindkey` listing (`^C`, `\\e[A`, …).
#[must_use]
pub fn format_key(keys: &[u8]) -> String {
    if keys.is_empty() {
        return String::new();
    }
    if keys.len() == 1 {
        return format_byte(keys[0]);
    }
    let mut out = String::new();
    for &byte in keys {
        out.push_str(&format_byte(byte));
    }
    out
}

fn format_byte(byte: u8) -> String {
    match byte {
        b'\t' => r"\t".to_owned(),
        b'\r' => r"\r".to_owned(),
        b'\n' => r"\n".to_owned(),
        0x1b => r"\e".to_owned(),
        0x7f => "^?".to_owned(),
        b if b < 0x20 => format!("^{}", (b + b'@') as char),
        b if b.is_ascii_graphic() || b == b' ' => (b as char).to_string(),
        b => format!(r"\x{b:02x}"),
    }
}
