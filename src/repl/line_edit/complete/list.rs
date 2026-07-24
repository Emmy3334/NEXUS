//! Column layout for ambiguous Tab completion listings.

/// Soft cap so a huge match set does not flood the TTY (zsh `LISTMAX`-ish).
pub(super) const LIST_MAX: usize = 100;

/// Format matches into terminal-width columns (column-major, like `ls`).
pub fn format_columns(items: &[String], width: usize) -> Vec<String> {
    if items.is_empty() {
        return Vec::new();
    }
    let width = width.max(1);
    let cell = items
        .iter()
        .map(|s| s.chars().count())
        .max()
        .unwrap_or(1)
        .max(1)
        + 2;
    let n_cols = (width / cell).max(1);
    let n_rows = items.len().div_ceil(n_cols);
    let mut lines = Vec::with_capacity(n_rows);
    for row in 0..n_rows {
        lines.push(format_row(items, row, n_rows, n_cols, cell));
    }
    lines
}

/// Cap + column-format for TTY display; uses `$COLUMNS` (default 80).
pub fn list_display_lines(matches: &[String]) -> Vec<String> {
    let width = std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80)
        .max(1);
    list_display_lines_width(matches, width)
}

/// Same as [`list_display_lines`] with an explicit terminal width (tests).
pub fn list_display_lines_width(matches: &[String], width: usize) -> Vec<String> {
    let (shown, omitted) = truncate(matches);
    let mut lines = format_columns(shown, width);
    if omitted > 0 {
        lines.push(format!("... and {omitted} more"));
    }
    lines
}

/// Vertical menu lines with reverse-video on `highlight` (arrow menu-select).
pub fn list_menu_lines(matches: &[String], highlight: usize) -> Vec<String> {
    let (shown, omitted) = truncate(matches);
    let mut lines = Vec::with_capacity(shown.len() + usize::from(omitted > 0));
    for (i, item) in shown.iter().enumerate() {
        lines.push(highlight_line(item, i == highlight));
    }
    if omitted > 0 {
        lines.push(format!("... and {omitted} more"));
    }
    lines
}

fn highlight_line(text: &str, on: bool) -> String {
    if on {
        format!("\x1b[7m{text}\x1b[0m")
    } else {
        text.to_owned()
    }
}

fn truncate(matches: &[String]) -> (&[String], usize) {
    if matches.len() > LIST_MAX {
        (&matches[..LIST_MAX], matches.len() - LIST_MAX)
    } else {
        (matches, 0)
    }
}

fn format_row(items: &[String], row: usize, n_rows: usize, n_cols: usize, cell: usize) -> String {
    let mut line = String::new();
    for col in 0..n_cols {
        let idx = col * n_rows + row;
        if idx >= items.len() {
            break;
        }
        let item = &items[idx];
        if col + 1 < n_cols && idx + n_rows < items.len() {
            line.push_str(&format!("{item:<width$}", width = cell));
        } else {
            line.push_str(item);
        }
    }
    line
}
