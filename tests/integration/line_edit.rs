//! Line-edition coverage: quote continuation, rebinding, and history recall.

use nexus::history::History;
use nexus::repl::{self, Action, HistoryRecall, KeyBindings};
use std::io::Cursor;

#[test]
fn interactive_continues_unclosed_double_quote() {
    let path =
        std::env::temp_dir().join(format!("nexus_line_edit_quote_{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let input = format!("printf '%s' \"hi\nthere\" > {}\n", path.display());
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, true).unwrap();
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    assert!(stderr.is_empty(), "{}", String::from_utf8_lossy(&stderr));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "hi\nthere");
    let _ = std::fs::remove_file(path);
}

#[test]
fn key_bindings_can_be_rebound() {
    let mut bindings = KeyBindings::new();
    bindings.bind(b"\x18".to_vec(), Action::Interrupt);
    assert_eq!(bindings.lookup(b"\x18"), Some(Action::Interrupt));
    assert_eq!(bindings.lookup(b"\t"), Some(Action::Complete));
}

#[test]
fn noninteractive_unclosed_quote_still_errors() {
    let mut stdin = Cursor::new("echo \"hi\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 1);
    assert!(String::from_utf8(stderr)
        .unwrap()
        .contains("Unmatched quote"));
}

#[test]
fn interactive_blank_line_is_not_eof() {
    let mut stdin = Cursor::new("\nexit 7\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, true).unwrap();
    assert_eq!(code, 7, "stderr={}", String::from_utf8_lossy(&stderr));
}

#[test]
fn history_recall_walks_older_and_restores_draft() {
    let mut history = History::default();
    history.push("first");
    history.push("second");
    let mut nav = HistoryRecall::new(&history);
    let mut line = String::from("draft");

    nav.older(&mut line);
    assert_eq!(line, "second");
    assert_eq!(nav.offset(), 1);

    nav.older(&mut line);
    assert_eq!(line, "first");
    assert_eq!(nav.offset(), 2);

    nav.older(&mut line);
    assert_eq!(line, "first");
    assert_eq!(nav.offset(), 2);

    nav.newer(&mut line);
    assert_eq!(line, "second");
    nav.newer(&mut line);
    assert_eq!(line, "draft");
    assert_eq!(nav.offset(), 0);
}

#[test]
fn history_recall_newer_on_draft_is_noop() {
    let history = History::default();
    let mut nav = HistoryRecall::new(&history);
    let mut line = String::from("only");
    nav.newer(&mut line);
    assert_eq!(line, "only");
    assert_eq!(nav.offset(), 0);
}
