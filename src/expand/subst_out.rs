//! Splice command-substitution stdout into field builder(s).

use super::fields::FieldBuilder;

/// Insert `output` into `fields` (split on whitespace when `split_words`).
pub(super) fn apply(fields: &mut FieldBuilder, output: &str, split_words: bool) {
    if split_words {
        splice_split(fields, output);
    } else {
        let joined = output.replace(['\n', '\r'], " ");
        fields.current().push_str_literal(&joined);
    }
}

fn splice_split(fields: &mut FieldBuilder, output: &str) {
    let pieces: Vec<&str> = output.split_whitespace().collect();
    if pieces.is_empty() {
        return;
    }
    fields.current().push_str_globable(pieces[0]);
    for piece in &pieces[1..] {
        fields.start_field();
        fields.current().push_str_globable(piece);
    }
}
