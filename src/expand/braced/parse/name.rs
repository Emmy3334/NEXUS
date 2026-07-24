//! Parameter name scanning in braced expansions.

use super::super::super::name::{is_name_continue, is_name_start};

pub(super) fn param_name_end(body: &str) -> usize {
    let mut chars = body.char_indices();
    let Some((_, first)) = chars.next() else {
        return 0;
    };
    if first == '*' || first == '#' {
        return first.len_utf8();
    }
    if first.is_ascii_digit() {
        return scan_while(body, first.len_utf8(), |c| c.is_ascii_digit());
    }
    if is_name_start(first) {
        return scan_while(body, first.len_utf8(), is_name_continue);
    }
    0
}

fn scan_while(body: &str, start: usize, ok: impl Fn(char) -> bool) -> usize {
    let mut end = start;
    for (i, c) in body[start..].char_indices() {
        if !ok(c) {
            break;
        }
        end = start + i + c.len_utf8();
    }
    end
}
