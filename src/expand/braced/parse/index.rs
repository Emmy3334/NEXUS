//! `${name[n]}`, `${name[@]}`, and `${name[*]}` parse forms.

use super::Form;

pub(super) fn index_form<'a>(name: &'a str, rest: &'a str) -> Option<Form<'a>> {
    if !rest.starts_with('[') {
        return None;
    }
    let close = rest.find(']')?;
    if !rest[close + 1..].is_empty() {
        return None;
    }
    let index = &rest[1..close];
    if index == "@" || index == "*" {
        return Some(Form::Index { name, index });
    }
    if !index.is_empty() && index.chars().all(|c| c.is_ascii_digit()) {
        return Some(Form::Index { name, index });
    }
    None
}
