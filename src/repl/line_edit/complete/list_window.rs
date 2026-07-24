//! Fit completion menu lines into a TTY-safe row budget (avoid viewport scroll).

use super::list_tagged::list_menu_lines_tagged;
use super::match_item::Match;

/// Tagged menu lines clipped to `max_rows`, scrolled so the highlight stays visible.
///
/// Writing more rows than fit below the prompt scrolls the viewport and breaks
/// relative cursor restore — keep this under the terminal height.
pub fn list_menu_fit(matches: &[Match], highlight: usize, max_rows: usize) -> Vec<String> {
    let lines = list_menu_lines_tagged(matches, highlight);
    window(&lines, max_rows.max(1))
}

fn window(lines: &[String], max_rows: usize) -> Vec<String> {
    if lines.len() <= max_rows {
        return lines.to_vec();
    }
    let body = max_rows.saturating_sub(1).max(1);
    let hl = highlight_row(lines);
    let mut start = hl.saturating_sub(body / 2);
    if start + body > lines.len() {
        start = lines.len() - body;
    }
    let end = start + body;
    let mut out = lines[start..end].to_vec();
    out.push(format!("... ({} more)", lines.len() - body));
    out
}

fn highlight_row(lines: &[String]) -> usize {
    lines
        .iter()
        .position(|l| l.contains("\x1b[7m"))
        .unwrap_or(0)
}
