//! Token bounds and common-prefix helpers for Tab completion.

pub(super) fn token_at(buffer: &str, cursor: usize) -> (usize, String) {
    let bytes = buffer.as_bytes();
    let mut start = cursor.min(bytes.len());
    while start > 0 && !bytes[start - 1].is_ascii_whitespace() {
        start -= 1;
    }
    (start, buffer[start..cursor.min(buffer.len())].to_owned())
}

pub(super) fn apply_match(buffer: &mut String, cursor: &mut usize, start: usize, value: &str) {
    let end = (*cursor).min(buffer.len());
    buffer.replace_range(start..end, value);
    *cursor = start + value.len();
}

pub(super) fn common_prefix(items: &[String]) -> Option<String> {
    let first = items.first()?;
    let mut end = first.len();
    for item in &items[1..] {
        end = end.min(
            first
                .chars()
                .zip(item.chars())
                .take_while(|(a, b)| a == b)
                .count(),
        );
    }
    Some(first.chars().take(end).collect())
}
