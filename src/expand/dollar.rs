//! `$` / `$?` / `$status` / `$n` / `$#` / `$*` / `${…}` into an [`ExpandedWord`].

use super::braced;
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
        Some('#') => {
            chars.next();
            push_argc(env, out);
        }
        Some('*') => {
            chars.next();
            out.push_str_literal(&env.star());
        }
        Some('{') => braced::push_braced(chars, env, last_status, out, globable),
        Some(c) if c.is_ascii_digit() => push_digits(chars, env, out, globable),
        Some(c) if is_name_start(c) => push_plain_name(chars, env, last_status, out, globable),
        _ => out.push_literal('$'),
    }
}

fn push_status(last_status: u8, out: &mut ExpandedWord) {
    let mut buf = String::new();
    let _ = std::fmt::Write::write_fmt(&mut buf, format_args!("{last_status}"));
    out.push_str_literal(&buf);
}

fn push_argc(env: &ShellEnvironment, out: &mut ExpandedWord) {
    let mut buf = String::new();
    let _ = std::fmt::Write::write_fmt(&mut buf, format_args!("{}", env.argc()));
    out.push_str_literal(&buf);
}

fn push_digits(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    env: &ShellEnvironment,
    out: &mut ExpandedWord,
    globable: bool,
) {
    let mut name = String::new();
    while let Some(c) = chars.peek().copied() {
        if !c.is_ascii_digit() {
            break;
        }
        name.push(c);
        chars.next();
    }
    push_named_parameter(&name, env, 0, out, globable);
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
