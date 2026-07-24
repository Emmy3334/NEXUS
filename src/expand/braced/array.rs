//! `${name[i]}`, `${name[@]}`, `${name[*]}`, and associative-array forms.

use super::super::fields::FieldBuilder;
use super::value;
use crate::env::ShellEnvironment;

/// Expand an index form into one or more fields (array or associative).
pub(super) fn index(
    name: &str,
    index: &str,
    env: &ShellEnvironment,
    fields: &mut FieldBuilder,
    globable: bool,
) {
    if env.is_assoc(name) {
        return assoc_index(name, index, env, fields, globable);
    }
    let Some(elements) = env.array_get(name) else {
        return;
    };
    match index {
        "@" => splat(elements, fields, globable),
        "*" => join_push(elements, fields, globable),
        digits => push_element(elements, digits, fields, globable),
    }
}

/// Expand `${(k)name}` into the sorted keys of an associative array.
pub(super) fn keys(name: &str, env: &ShellEnvironment, fields: &mut FieldBuilder, globable: bool) {
    splat(&env.assoc_keys(name), fields, globable);
}

/// Expand `${(v)name}` into the values of an associative array (key order).
pub(super) fn values(
    name: &str,
    env: &ShellEnvironment,
    fields: &mut FieldBuilder,
    globable: bool,
) {
    splat(&env.assoc_values(name), fields, globable);
}

fn assoc_index(
    name: &str,
    index: &str,
    env: &ShellEnvironment,
    fields: &mut FieldBuilder,
    globable: bool,
) {
    match index {
        "@" => splat(&env.assoc_values(name), fields, globable),
        "*" => join_push(&env.assoc_values(name), fields, globable),
        key => {
            fields.clear_elide();
            let text = env.assoc_get(name, key).unwrap_or_default();
            value::push_text(text, fields.current(), globable);
        }
    }
}

fn splat(items: &[String], fields: &mut FieldBuilder, globable: bool) {
    if items.is_empty() {
        fields.mark_empty_at();
        return;
    }
    fields.clear_elide();
    value::push_text(&items[0], fields.current(), globable);
    for item in &items[1..] {
        fields.start_field();
        value::push_text(item, fields.current(), globable);
    }
}

fn join_push(items: &[String], fields: &mut FieldBuilder, globable: bool) {
    fields.clear_elide();
    value::push_text(&items.join(" "), fields.current(), globable);
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
