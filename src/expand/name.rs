//! Named parameter lookup (`$name` / `${name}` / `$status` / `$n` / `$#`).

use super::ExpandedWord;
use crate::env::ShellEnvironment;

pub(super) fn push_named_parameter(
    name: &str,
    env: &mut ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
    globable: bool,
) {
    if name == "status" {
        push_text(&format!("{last_status}"), out, false);
        return;
    }
    if name == "#" {
        push_text(&format!("{}", env.argc()), out, false);
        return;
    }
    if name == "*" {
        push_text(&env.star(), out, globable);
        return;
    }
    if let Ok(index) = name.parse::<usize>() {
        if let Some(value) = env.positional(index) {
            push_text(value, out, globable);
        }
        return;
    }
    let Some(value) = env.lookup(name) else {
        return;
    };
    push_text(value, out, globable);
}

fn push_text(value: &str, out: &mut ExpandedWord, globable: bool) {
    if globable {
        out.push_str_globable(value);
    } else {
        out.push_str_literal(value);
    }
}

pub(super) fn is_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

pub(super) fn is_name_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
