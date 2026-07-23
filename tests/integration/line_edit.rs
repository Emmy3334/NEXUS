//! Line-edition coverage: quote continuation, rebinding, and history recall.

use nexus::history::History;
use nexus::keybind::{Action, Binding, KeyBindings};
use nexus::repl::{self, HistoryRecall};
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
    bindings.bind(b"\x18".to_vec(), Binding::Action(Action::Interrupt), false);
    assert_eq!(
        bindings.lookup(b"\x18"),
        Some(&Binding::Action(Action::Interrupt))
    );
    assert_eq!(
        bindings.lookup(b"\t"),
        Some(&Binding::Action(Action::Complete))
    );
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

#[cfg(unix)]
#[test]
fn paste_buffer_peels_complete_lines_and_keeps_tail() {
    use std::collections::VecDeque;

    let mut q = VecDeque::from(b"echo a\necho b\r\npartial".to_vec());
    assert_eq!(repl::take_complete_line(&mut q).as_deref(), Some("echo a"));
    assert_eq!(repl::take_complete_line(&mut q).as_deref(), Some("echo b"));
    assert_eq!(repl::take_complete_line(&mut q), None);
    assert_eq!(Vec::from(q), b"partial");
}

#[cfg(unix)]
#[test]
fn paste_buffer_handles_cr_only_and_empty_lines() {
    use std::collections::VecDeque;

    let mut q = VecDeque::from(b"one\r\ntwo\r\n\nthree\n".to_vec());
    assert_eq!(repl::take_complete_line(&mut q).as_deref(), Some("one"));
    assert_eq!(repl::take_complete_line(&mut q).as_deref(), Some("two"));
    assert_eq!(repl::take_complete_line(&mut q).as_deref(), Some(""));
    assert_eq!(repl::take_complete_line(&mut q).as_deref(), Some("three"));
    assert_eq!(repl::take_complete_line(&mut q), None);
}

#[cfg(unix)]
#[test]
fn multiline_accept_queues_remaining_lines_in_order() {
    use std::collections::VecDeque;

    // Accept peels the first line and terminates the tail so every segment drains.
    let mut q = VecDeque::new();
    let text = "set rlimit=1\nset path_jail=0\nset rlimit_cpu=5";
    let (first, rest) = text.split_once('\n').unwrap();
    assert_eq!(first, "set rlimit=1");
    if !(rest.ends_with('\n') || rest.ends_with('\r')) {
        q.push_front(b'\n');
    }
    for &b in rest.as_bytes().iter().rev() {
        q.push_front(b);
    }
    assert_eq!(
        repl::take_complete_line(&mut q).as_deref(),
        Some("set path_jail=0")
    );
    assert_eq!(
        repl::take_complete_line(&mut q).as_deref(),
        Some("set rlimit_cpu=5")
    );
    assert_eq!(repl::take_complete_line(&mut q), None);
    assert!(q.is_empty());
}
