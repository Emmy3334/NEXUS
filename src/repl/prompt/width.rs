//! Visible terminal columns for strings that may contain ANSI CSI sequences.

use unicode_width::UnicodeWidthChar;

/// Count display columns, ignoring CSI/SGR escape sequences.
#[must_use]
pub fn visible_columns(text: &str) -> usize {
    let mut cols = 0usize;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            skip_csi(&mut chars);
            continue;
        }
        cols = cols.saturating_add(UnicodeWidthChar::width(ch).unwrap_or(0));
    }
    cols
}

fn skip_csi(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    match chars.next() {
        Some('[') => {
            for c in chars.by_ref() {
                if ('\x40'..='\x7e').contains(&c) {
                    break;
                }
            }
        }
        Some(']') => skip_osc(chars),
        _ => {}
    }
}

fn skip_osc(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while let Some(c) = chars.next() {
        if c == '\u{7}' {
            break;
        }
        if c == '\u{1b}' && chars.peek() == Some(&'\\') {
            chars.next();
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::visible_columns;

    #[test]
    fn plain_ascii() {
        assert_eq!(visible_columns("$> "), 3);
    }

    #[test]
    fn strips_sgr() {
        assert_eq!(visible_columns("\x1b[32mhi\x1b[0m"), 2);
    }
}
