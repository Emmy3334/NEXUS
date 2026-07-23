//! Pad columns into a kubectl-like table.

#[must_use]
pub(super) fn render(headers: &[&str], rows: &[Vec<String>]) -> Vec<String> {
    let cols = headers.len();
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate().take(cols) {
            widths[i] = widths[i].max(cell.len());
        }
    }
    let mut out = Vec::with_capacity(rows.len() + 1);
    out.push(format_row(
        &headers.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>(),
        &widths,
    ));
    for row in rows {
        out.push(format_row(row, &widths));
    }
    out
}

fn format_row(cells: &[String], widths: &[usize]) -> String {
    let mut line = String::new();
    for (i, cell) in cells.iter().enumerate() {
        if i > 0 {
            line.push_str("   ");
        }
        let width = widths.get(i).copied().unwrap_or(cell.len());
        line.push_str(&format!("{cell:<width$}"));
    }
    line.trim_end().to_owned()
}
