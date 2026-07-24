//! `${name[n]}`, `${name[@]}`, and `${name[*]}` parse forms.

use super::Form;

pub(super) fn index_form<'a>(name: &'a str, rest: &'a str) -> Option<Form<'a>> {
    if !rest.starts_with('[') {
        return None;
    }
    let close = rest.rfind(']')?;
    if !rest[close + 1..].is_empty() {
        return None;
    }
    let index = &rest[1..close];
    if index.is_empty() {
        return None;
    }
    // `@` / `*` / numeric select array elements; any other key selects an
    // associative-array value (resolved against the live env at expand time).
    Some(Form::Index { name, index })
}
