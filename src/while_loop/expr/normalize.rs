//! Merge lex-split `&&` / `||` (and `<=` / `>=`) into single operators.

pub(super) fn merge_ops(parts: Vec<String>) -> Vec<String> {
    let mut out = Vec::with_capacity(parts.len());
    let mut i = 0;
    while i < parts.len() {
        if let Some(merged) = two_char_op(&parts, i) {
            out.push(merged);
            i += 2;
        } else {
            out.push(parts[i].clone());
            i += 1;
        }
    }
    out
}

fn two_char_op(parts: &[String], i: usize) -> Option<String> {
    let next = parts.get(i + 1).map(String::as_str)?;
    match (parts[i].as_str(), next) {
        ("&", "&") => Some("&&".into()),
        ("|", "|") => Some("||".into()),
        ("<", "=") => Some("<=".into()),
        (">", "=") => Some(">=".into()),
        _ => None,
    }
}
