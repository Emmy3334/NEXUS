//! Read a braced parameter body with nested `{` / `}` depth.

pub(super) fn body(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> (String, bool) {
    let mut out = String::new();
    let mut depth = 1usize;
    for ch in chars.by_ref() {
        match ch {
            '{' => {
                depth += 1;
                out.push(ch);
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return (out, true);
                }
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    (out, false)
}
