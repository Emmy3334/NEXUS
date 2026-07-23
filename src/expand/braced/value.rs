//! Resolve parameter values and splice expanded operator words.

use super::super::decode::expand_word_for_exec;
use super::super::name::push_named_parameter;
use super::super::ExpandedWord;
use crate::env::ShellEnvironment;

pub(super) fn resolve(name: &str, env: &ShellEnvironment, last_status: u8) -> String {
    let mut buf = ExpandedWord::default();
    push_named_parameter(name, env, last_status, &mut buf, false);
    buf.into_string()
}

pub(super) fn is_set_nonempty(name: &str, env: &ShellEnvironment, last_status: u8) -> bool {
    !resolve(name, env, last_status).is_empty()
}

pub(super) fn push_expanded_word(
    word: &str,
    env: &ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
    globable: bool,
) {
    match expand_word_for_exec(word, env, last_status) {
        Ok(expanded) => push_text(&expanded.into_string(), out, globable),
        Err(_) => push_text(word, out, globable),
    }
}

pub(super) fn expand_pattern(pat: &str, env: &ShellEnvironment, last_status: u8) -> String {
    expand_word_for_exec(pat, env, last_status)
        .map(ExpandedWord::into_string)
        .unwrap_or_else(|_| pat.to_owned())
}

pub(super) fn push_text(value: &str, out: &mut ExpandedWord, globable: bool) {
    if globable {
        out.push_str_globable(value);
    } else {
        out.push_str_literal(value);
    }
}
