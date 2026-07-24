//! `${name:offset}` and `${name:offset:length}` parse forms.
//!
//! `:-` / `:+` are never slices (those are default/alternate). Negative offsets
//! use a space after the colon, bash-style: `${name: -2}`.

use super::Form;

pub(super) fn slice_form<'a>(name: &'a str, rest: &'a str) -> Option<Form<'a>> {
    if !rest.starts_with(':') || rest.starts_with(":-") || rest.starts_with(":+") {
        return None;
    }
    let tail = rest[1..].trim_start();
    let (offset_end, after_offset) = signed_number_end(tail)?;
    let offset = &tail[..offset_end];
    if after_offset.is_empty() {
        return Some(Form::Slice {
            name,
            offset,
            length: None,
        });
    }
    let after_offset = after_offset.strip_prefix(':')?;
    let (len_end, rest2) = unsigned_number_end(after_offset)?;
    if !rest2.is_empty() || len_end == 0 {
        return None;
    }
    Some(Form::Slice {
        name,
        offset,
        length: Some(&after_offset[..len_end]),
    })
}

fn signed_number_end(input: &str) -> Option<(usize, &str)> {
    let rest = input.strip_prefix('-').unwrap_or(input);
    let end = digit_run_len(rest);
    if end == 0 {
        return None;
    }
    let total = if input.starts_with('-') { end + 1 } else { end };
    Some((total, &input[total..]))
}

fn unsigned_number_end(input: &str) -> Option<(usize, &str)> {
    let end = digit_run_len(input);
    Some((end, &input[end..]))
}

fn digit_run_len(input: &str) -> usize {
    input
        .char_indices()
        .find(|(_, c)| !c.is_ascii_digit())
        .map(|(i, _)| i)
        .unwrap_or(input.len())
}
