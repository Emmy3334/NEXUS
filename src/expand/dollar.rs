//! `$` / `$?` / `$status` / `${name}` parameter expansion into an [`ExpandedWord`].

use super::name::{is_name_continue, is_name_start, push_named_parameter};
use super::ExpandedWord;
use crate::env::ShellEnvironment;

pub(super) fn push_parameter(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    env: &ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
    globable: bool,
) {
    match chars.peek().copied() {
        Some('?') => {
            chars.next();
            push_status(last_status, out);
        }
        Some('{') => push_braced(chars, env, last_status, out, globable),
        Some(c) if is_name_start(c) => push_plain_name(chars, env, last_status, out, globable),
        _ => out.push_literal('$'),
    }
}

fn push_status(last_status: u8, out: &mut ExpandedWord) {
    let mut buf = String::new();
    let _ = std::fmt::Write::write_fmt(&mut buf, format_args!("{last_status}"));
    out.push_str_literal(&buf);
}

fn push_braced(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    env: &ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
    globable: bool,
) {
    chars.next();
    let (name, closed) = read_braced_name(chars);
    if !closed {
        out.push_literal('$');
        out.push_literal('{');
        out.push_str_literal(&name);
        return;
    }
    if name.is_empty() {
        return;
    }
    push_named_parameter(&name, env, last_status, out, globable);
}

fn read_braced_name(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> (String, bool) {
    let mut name = String::new();
    for ch in chars.by_ref() {
        if ch == '}' {
            return (name, true);
        }
        name.push(ch);
    }
    (name, false)
}

fn push_plain_name(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    env: &ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
    globable: bool,
) {
    let Some(first) = chars.next() else {
        return;
    };
    let mut name = String::new();
    name.push(first);
    while let Some(c) = chars.peek().copied() {
        if !is_name_continue(c) {
            break;
        }
        name.push(c);
        chars.next();
    }
    push_named_parameter(&name, env, last_status, out, globable);
}
