//! Apply braced parameter operators to an [`ExpandedWord`].

use super::super::name::push_named_parameter;
use super::super::ExpandedWord;
use super::trim;
use super::value;
use crate::env::ShellEnvironment;

pub(super) fn length(name: &str, env: &ShellEnvironment, last_status: u8, out: &mut ExpandedWord) {
    let value = value::resolve(name, env, last_status);
    let n = value.chars().count();
    out.push_str_literal(&n.to_string());
}

pub(super) fn with_default(
    name: &str,
    word: &str,
    env: &ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
    globable: bool,
) {
    if value::is_set_nonempty(name, env, last_status) {
        push_named_parameter(name, env, last_status, out, globable);
        return;
    }
    value::push_expanded_word(word, env, last_status, out, globable);
}

pub(super) fn alternate(
    name: &str,
    word: &str,
    env: &ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
    globable: bool,
) {
    if !value::is_set_nonempty(name, env, last_status) {
        return;
    }
    value::push_expanded_word(word, env, last_status, out, globable);
}

pub(super) fn strip_prefix(
    name: &str,
    pat: &str,
    longest: bool,
    env: &ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
    globable: bool,
) {
    let text = value::resolve(name, env, last_status);
    let pattern = value::expand_pattern(pat, env, last_status);
    value::push_text(&trim::prefix(&text, &pattern, longest), out, globable);
}

pub(super) fn strip_suffix(
    name: &str,
    pat: &str,
    longest: bool,
    env: &ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
    globable: bool,
) {
    let text = value::resolve(name, env, last_status);
    let pattern = value::expand_pattern(pat, env, last_status);
    value::push_text(&trim::suffix(&text, &pattern, longest), out, globable);
}
