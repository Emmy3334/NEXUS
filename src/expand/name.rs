//! Named parameter lookup (`$name` / `${name}` / `$status`).

use super::ExpandedWord;
use crate::env::ShellEnvironment;

pub(super) fn push_named_parameter(
    name: &str,
    env: &ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
    globable: bool,
) {
    if name == "status" {
        let mut buf = String::new();
        let _ = std::fmt::Write::write_fmt(&mut buf, format_args!("{last_status}"));
        out.push_str_literal(&buf);
        return;
    }
    let Some(value) = env.lookup(name) else {
        return;
    };
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
