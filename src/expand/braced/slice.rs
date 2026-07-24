//! `${name:offset}` and `${name:offset:length}` substring expansion.

use super::value;
use super::ExpandedWord;
use crate::env::ShellEnvironment;

pub(super) fn slice(
    name: &str,
    offset: &str,
    length: Option<&str>,
    env: &mut ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
    globable: bool,
) {
    let text = value::resolve(name, env, last_status);
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let start = resolve_offset(offset, len);
    let end = match length {
        None => len,
        Some("") => start,
        Some(n) => start.saturating_add(parse_usize(n)).min(len),
    };
    let piece: String = chars.get(start..end).unwrap_or(&[]).iter().collect();
    value::push_text(&piece, out, globable);
}

fn resolve_offset(raw: &str, len: usize) -> usize {
    let Ok(mut n) = parse_signed(raw) else {
        return 0;
    };
    if n < 0 {
        n += len as i64;
        if n < 0 {
            return 0;
        }
    }
    (n as usize).min(len)
}

fn parse_signed(raw: &str) -> Result<i64, ()> {
    let (sign, digits) = if let Some(rest) = raw.strip_prefix('-') {
        (-1_i64, rest)
    } else {
        (1, raw)
    };
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err(());
    }
    let n: i64 = digits.parse().map_err(|_| ())?;
    Ok(sign * n)
}

fn parse_usize(raw: &str) -> usize {
    if raw.is_empty() || !raw.chars().all(|c| c.is_ascii_digit()) {
        return 0;
    }
    raw.parse().unwrap_or(0)
}
