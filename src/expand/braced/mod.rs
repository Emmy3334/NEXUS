//! Nested `${…}` body reader and operator dispatch.

mod apply;
mod parse;
mod read;
mod trim;
mod value;

use super::name::push_named_parameter;
use super::ExpandedWord;
use crate::env::ShellEnvironment;

use self::parse::Form;

pub(super) fn push_braced(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
    globable: bool,
) {
    chars.next();
    let (body, closed) = read::body(chars);
    if !closed {
        out.push_literal('$');
        out.push_literal('{');
        out.push_str_literal(&body);
        return;
    }
    if body.is_empty() {
        return;
    }
    dispatch(&body, env, last_status, out, globable);
}

fn dispatch(
    body: &str,
    env: &mut ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
    globable: bool,
) {
    match parse::form(body) {
        Form::Plain(name) => push_named_parameter(name, env, last_status, out, globable),
        Form::Length(name) => apply::length(name, env, last_status, out),
        Form::Default { name, word } => {
            apply::with_default(name, word, env, last_status, out, globable);
        }
        Form::Alternate { name, word } => {
            apply::alternate(name, word, env, last_status, out, globable);
        }
        Form::StripPrefix { name, pat, longest } => {
            apply::strip_prefix(name, pat, longest, env, last_status, out, globable)
        }
        Form::StripSuffix { name, pat, longest } => {
            apply::strip_suffix(name, pat, longest, env, last_status, out, globable)
        }
    }
}
