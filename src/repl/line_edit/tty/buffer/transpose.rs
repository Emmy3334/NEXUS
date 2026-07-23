//! Transpose adjacent whitespace-separated words.

use super::word::{backward_word_start, forward_word_end};
use super::EditBuffer;

pub(in crate::repl::line_edit::tty) fn transpose_words(edit: &mut EditBuffer) {
    let text = &edit.text;
    let end2 = forward_word_end(text, edit.cursor);
    let start2 = backward_word_start(text, end2);
    if start2 >= end2 {
        return;
    }
    let start1 = backward_word_start(text, start2);
    let end1 = forward_word_end(text, start1);
    if end1 > start2 || start1 >= end1 {
        return;
    }
    let left = text[start1..end1].to_owned();
    let mid = text[end1..start2].to_owned();
    let right = text[start2..end2].to_owned();
    if left.is_empty() || right.is_empty() {
        return;
    }
    let mut out = String::with_capacity(text.len());
    out.push_str(&text[..start1]);
    out.push_str(&right);
    out.push_str(&mid);
    out.push_str(&left);
    out.push_str(&text[end2..]);
    edit.cursor = start1 + right.len() + mid.len() + left.len();
    edit.text = out;
}
