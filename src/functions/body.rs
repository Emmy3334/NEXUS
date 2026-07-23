//! Brace-depth helpers for function bodies.

/// If `text` contains a matching close for an already-opened `{`, return
/// `(inner, remainder_after_close)`.
#[must_use]
pub(crate) fn split_closed(text: &str) -> Option<(String, String)> {
    let mut depth = 1i32;
    for (i, ch) in text.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let inner = text[..i].to_owned();
                    let rest = text[i + ch.len_utf8()..].to_owned();
                    return Some((inner, rest));
                }
            }
            _ => {}
        }
    }
    None
}

/// Update brace depth for one collected body line; returns new depth.
#[must_use]
pub(crate) fn depth_delta(line: &str) -> i32 {
    let mut delta = 0i32;
    for ch in line.chars() {
        match ch {
            '{' => delta += 1,
            '}' => delta -= 1,
            _ => {}
        }
    }
    delta
}

/// Strip a final closing `}` line (or trailing `}` on the last line).
pub(crate) fn finalize_body(mut lines: Vec<String>) -> String {
    if let Some(last) = lines.last_mut() {
        let trimmed = last.trim_end();
        if trimmed == "}" {
            lines.pop();
        } else if let Some(stripped) = trimmed.strip_suffix('}') {
            *last = stripped.trim_end().to_owned();
            if last.is_empty() {
                lines.pop();
            }
        }
    }
    lines.join("\n")
}
