//! Parse key strings and editor command names for `bindkey`.

use super::Action;

/// Parse a caret / escape key description into bytes.
pub fn parse_key(spec: &str) -> Option<Vec<u8>> {
    if spec.is_empty() {
        return None;
    }
    let bytes = spec.as_bytes();
    if bytes.len() == 2 && bytes[0] == b'^' {
        return Some(vec![ctrl_byte(bytes[1])?]);
    }
    if let Some(rest) = spec.strip_prefix(r"\e") {
        let mut out = vec![0x1b];
        out.extend(rest.as_bytes());
        return Some(out);
    }
    if let Some(rest) = spec.strip_prefix(r"\x") {
        return parse_hex_bytes(rest);
    }
    Some(spec.as_bytes().to_vec())
}

/// `-b` key forms: `^X`, `C-x`, `M-x`, `M-^X`.
pub fn parse_key_b(spec: &str) -> Option<Vec<u8>> {
    if let Some(rest) = strip_ci_prefix(spec, "M-") {
        let inner = parse_key_b(rest)?;
        let mut out = vec![0x1b];
        out.extend(inner);
        return Some(out);
    }
    if let Some(rest) = strip_ci_prefix(spec, "C-") {
        if rest.chars().count() != 1 {
            return None;
        }
        let ch = rest.chars().next()?.to_ascii_uppercase() as u8;
        return Some(vec![ctrl_byte(ch)?]);
    }
    parse_key(spec)
}

/// Map an editor command name to [`Action`].
pub fn parse_action(name: &str) -> Option<Action> {
    Some(match name {
        "accept-line" => Action::Accept,
        "backward-delete-char" => Action::Backspace,
        "delete-char" => Action::Delete,
        "backward-char" => Action::MoveLeft,
        "forward-char" => Action::MoveRight,
        "backward-word" => Action::MoveWordLeft,
        "forward-word" => Action::MoveWordRight,
        "kill-word" => Action::KillWordForward,
        "backward-kill-word" | "unix-word-rubout" => Action::KillWordBackward,
        "kill-line" => Action::KillToEol,
        "kill-whole-line" => Action::KillLine,
        "yank" => Action::Yank,
        "transpose-words" => Action::TransposeWords,
        "up-history" => Action::HistoryUp,
        "down-history" => Action::HistoryDown,
        "history-incremental-search-backward" => Action::HistoryISearch,
        "complete-word" => Action::Complete,
        "tty-sigintr" => Action::Interrupt,
        "end-of-file" => Action::Eof,
        "vi-cmd-mode" => Action::ViCmdMode,
        "vi-insert-mode" => Action::ViInsertMode,
        _ => return None,
    })
}

fn strip_ci_prefix<'a>(spec: &'a str, prefix: &str) -> Option<&'a str> {
    if spec.len() >= prefix.len() && spec[..prefix.len()].eq_ignore_ascii_case(prefix) {
        Some(&spec[prefix.len()..])
    } else {
        None
    }
}

fn ctrl_byte(ch: u8) -> Option<u8> {
    match ch {
        b'?' => Some(0x7f),
        b'@' => Some(0),
        b if b.is_ascii_uppercase() => Some(b - b'@'),
        b if b.is_ascii_lowercase() => Some(b - b'a' + 1),
        b if (b'['..=b'_').contains(&b) => Some(b - b'@'),
        _ => None,
    }
}

fn parse_hex_bytes(rest: &str) -> Option<Vec<u8>> {
    if rest.len() != 2 {
        return None;
    }
    let value = u8::from_str_radix(rest, 16).ok()?;
    Some(vec![value])
}
