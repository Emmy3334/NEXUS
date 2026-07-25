//! Fit completion candidates into a TTY-safe column grid (zsh-like listing).

use super::list::LIST_MAX;
use super::match_item::Match;

/// Gap between grid columns, matching [`super::list::format_columns`].
const GAP: usize = 2;

/// Column-major menu grid clipped to `max_rows`, highlight kept visible.
///
/// Writing more rows than fit below the prompt scrolls the viewport and breaks
/// relative cursor restore — keep this under the terminal height.
pub fn list_menu_fit(matches: &[Match], highlight: usize, max_rows: usize) -> Vec<String> {
    let width = std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80usize)
        .max(1);
    list_menu_fit_width(matches, highlight, max_rows, width)
}

/// Same as [`list_menu_fit`] with an explicit terminal width (tests).
pub fn list_menu_fit_width(
    matches: &[Match],
    highlight: usize,
    max_rows: usize,
    width: usize,
) -> Vec<String> {
    let lines = grid_menu_lines(matches, highlight, width.max(1));
    window(&lines, max_rows.max(1))
}

fn grid_menu_lines(matches: &[Match], highlight: usize, width: usize) -> Vec<String> {
    let shown = &matches[..matches.len().min(LIST_MAX)];
    let omitted = matches.len().saturating_sub(shown.len());
    if shown.is_empty() {
        return Vec::new();
    }
    let items: Vec<&str> = shown.iter().map(|m| m.value.as_str()).collect();
    let cell = items
        .iter()
        .map(|s| s.chars().count())
        .max()
        .unwrap_or(1)
        .max(1)
        + GAP;
    let n_cols = (width / cell).max(1);
    let n_rows = items.len().div_ceil(n_cols);
    let mut lines: Vec<String> = (0..n_rows)
        .map(|row| grid_row(&items, highlight, row, n_rows, n_cols, cell))
        .collect();
    if omitted > 0 {
        lines.push(format!("... and {omitted} more"));
    }
    lines
}

fn grid_row(
    items: &[&str],
    hl: usize,
    row: usize,
    n_rows: usize,
    n_cols: usize,
    cell: usize,
) -> String {
    let mut line = String::new();
    for col in 0..n_cols {
        let idx = col * n_rows + row;
        if idx >= items.len() {
            break;
        }
        let last = col + 1 == n_cols || idx + n_rows >= items.len();
        let field = if last {
            items[idx].chars().count()
        } else {
            cell
        };
        push_cell(&mut line, items[idx], idx == hl, field);
    }
    line
}

fn push_cell(line: &mut String, item: &str, on: bool, field: usize) {
    if on {
        line.push_str(&format!("\x1b[7m{item}\x1b[0m"));
    } else {
        line.push_str(item);
    }
    for _ in 0..field.saturating_sub(item.chars().count()) {
        line.push(' ');
    }
}

fn window(lines: &[String], max_rows: usize) -> Vec<String> {
    if lines.len() <= max_rows {
        return lines.to_vec();
    }
    let body = max_rows.saturating_sub(1).max(1);
    let hl = lines
        .iter()
        .position(|l| l.contains("\x1b[7m"))
        .unwrap_or(0);
    let mut start = hl.saturating_sub(body / 2);
    if start + body > lines.len() {
        start = lines.len() - body;
    }
    let mut out = lines[start..start + body].to_vec();
    out.push(format!("... ({} more)", lines.len() - body));
    out
}
