//! `$` / `$?` / `$n` / `$name` into field builder(s).

use super::arith;
use super::braced;
use super::fields::FieldBuilder;
use super::name::{is_name_continue, is_name_start, push_named_parameter};
use crate::env::ShellEnvironment;
use crate::lex::LexError;

pub(super) fn push_parameter(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
    fields: &mut FieldBuilder,
    globable: bool,
) -> Result<(), LexError> {
    match chars.peek().copied() {
        Some('?') => {
            chars.next();
            fields.clear_elide();
            push_status(last_status, fields.current());
        }
        Some('#') => {
            chars.next();
            fields.clear_elide();
            push_argc(env, fields.current());
        }
        Some('*') => {
            chars.next();
            fields.clear_elide();
            fields.current().push_str_literal(&env.star());
        }
        Some('{') => braced::push_braced(chars, env, last_status, fields, globable),
        Some('(') => {
            fields.clear_elide();
            push_arith_or_literal(chars, env, last_status, fields.current())?;
        }
        Some(c) if c.is_ascii_digit() => {
            fields.clear_elide();
            push_digits(chars, env, fields.current(), globable);
        }
        Some(c) if is_name_start(c) => {
            fields.clear_elide();
            push_plain_name(chars, env, last_status, fields.current(), globable);
        }
        _ => {
            fields.clear_elide();
            fields.current().push_literal('$');
        }
    }
    Ok(())
}

fn push_arith_or_literal(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
    out: &mut super::ExpandedWord,
) -> Result<(), LexError> {
    chars.next(); // '('
    if chars.peek() != Some(&'(') {
        out.push_literal('$');
        out.push_literal('(');
        return Ok(());
    }
    chars.next(); // second '('
    arith::push_arith(chars, env, last_status, out)
}

fn push_status(last_status: u8, out: &mut super::ExpandedWord) {
    let mut buf = String::new();
    let _ = std::fmt::Write::write_fmt(&mut buf, format_args!("{last_status}"));
    out.push_str_literal(&buf);
}

fn push_argc(env: &mut ShellEnvironment, out: &mut super::ExpandedWord) {
    let mut buf = String::new();
    let _ = std::fmt::Write::write_fmt(&mut buf, format_args!("{}", env.argc()));
    out.push_str_literal(&buf);
}

fn push_digits(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    env: &mut ShellEnvironment,
    out: &mut super::ExpandedWord,
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
    env: &mut ShellEnvironment,
    last_status: u8,
    out: &mut super::ExpandedWord,
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
