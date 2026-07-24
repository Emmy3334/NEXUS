//! `${name[i]}`, `${name[@]}`, and `${name[*]}` expansion.

use super::super::fields::FieldBuilder;
use super::super::ExpandedWord;
use super::value;
use crate::env::ShellEnvironment;

/// Expand an array index form into one or more fields.
pub(super) fn index(
    name: &str,
    index: &str,
    env: &ShellEnvironment,
    fields: &mut FieldBuilder,
    globable: bool,
) {
    let Some(elements) = env.array_get(name) else {
        return;
    };
    match index {
        "@" => splice_at(elements, fields, globable),
        "*" => {
            fields.clear_elide();
            value::push_text(&elements.join(" "), fields.current(), globable);
        }
        digits => push_element(elements, digits, fields, globable),
    }
}

fn splice_at(elements: &[String], fields: &mut FieldBuilder, globable: bool) {
    if elements.is_empty() {
        fields.mark_empty_at();
        return;
    }
    fields.clear_elide();
    push_one(&elements[0], fields.current(), globable);
    for elem in &elements[1..] {
        fields.start_field();
        push_one(elem, fields.current(), globable);
    }
}

fn push_element(elements: &[String], digits: &str, fields: &mut FieldBuilder, globable: bool) {
    let Ok(n) = digits.parse::<usize>() else {
        return;
    };
    if n == 0 {
        return;
    }
    fields.clear_elide();
    let text = elements.get(n - 1).cloned().unwrap_or_default();
    value::push_text(&text, fields.current(), globable);
}

fn push_one(text: &str, out: &mut ExpandedWord, globable: bool) {
    value::push_text(text, out, globable);
}
