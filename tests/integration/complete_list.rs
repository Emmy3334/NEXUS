//! Ambiguous completion column listing.

use nexus::repl::{format_columns, list_display_lines, list_display_lines_width};

#[test]
fn columns_are_column_major() {
    let items = ["a", "b", "c", "d", "e"]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    // cell width = 1 + 2 = 3 → 9/3 = 3 cols → 2 rows
    let lines = format_columns(&items, 9);
    assert_eq!(lines, vec!["a  c  e".to_string(), "b  d".to_string()]);
}

#[test]
fn narrow_terminal_falls_back_to_one_column() {
    let items = ["alpha", "beta"].map(str::to_owned).to_vec();
    let lines = format_columns(&items, 4);
    assert_eq!(lines, vec!["alpha".to_string(), "beta".to_string()]);
}

#[test]
fn list_caps_and_notes_omitted() {
    let items = (0..105).map(|i| format!("n{i:03}")).collect::<Vec<_>>();
    let lines = list_display_lines_width(&items, 40);
    assert!(lines.last().is_some_and(|l| l == "... and 5 more"));
    assert_eq!(lines.len(), format_columns(&items[..100], 40).len() + 1);
}

#[test]
fn empty_list_yields_no_lines() {
    assert!(format_columns(&[], 80).is_empty());
    assert!(list_display_lines(&[]).is_empty());
}

#[test]
fn menu_lines_highlight_selected_row() {
    use nexus::repl::list_menu_lines;
    let items = ["alpha", "beta"].map(str::to_owned).to_vec();
    let lines = list_menu_lines(&items, 1);
    assert_eq!(lines[0], "alpha");
    assert!(lines[1].contains("\x1b[7m"));
    assert!(lines[1].contains("beta"));
    assert!(lines[1].contains("\x1b[0m"));
}

#[test]
fn menu_fit_caps_rows_and_keeps_highlight() {
    use nexus::repl::{list_menu_fit, Match, Tag};
    let matches: Vec<Match> = (0..40)
        .map(|i| Match {
            value: format!("cmd{i:02}"),
            score: 0,
            tag: Tag::Commands,
            description: None,
        })
        .collect();
    let lines = list_menu_fit(&matches, 20, 8);
    assert!(lines.len() <= 8);
    assert!(lines.last().is_some_and(|l| l.contains("more")));
    assert!(lines
        .iter()
        .any(|l| l.contains("\x1b[7m") && l.contains("cmd20")));
}
