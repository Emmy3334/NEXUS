//! Reverse incremental history search (`HistoryISearch` / Ctrl-R).

use nexus::history::History;
use nexus::keybind::{action_name, parse_action, Action, Binding, KeyBindings};
use nexus::repl::HistoryISearch;

#[test]
fn ctrl_r_defaults_to_isearch() {
    let bindings = KeyBindings::new();
    assert_eq!(
        bindings.lookup(&[0x12]),
        Some(&Binding::Action(Action::HistoryISearch))
    );
}

#[test]
fn isearch_action_name_round_trip() {
    let name = "history-incremental-search-backward";
    assert_eq!(parse_action(name), Some(Action::HistoryISearch));
    assert_eq!(action_name(Action::HistoryISearch), name);
}

#[test]
fn isearch_refines_query_and_accepts_match() {
    let mut history = History::default();
    history.push("echo alpha");
    history.push("printf beta");
    history.push("echo gamma");

    let mut search = HistoryISearch::start(&history, "draft");
    assert_eq!(search.display_line(), "echo gamma");
    assert!(!search.failed);

    search.push_char('a');
    assert_eq!(search.display_line(), "echo gamma");
    search.push_char('l');
    assert_eq!(search.display_line(), "echo alpha");
    assert!(!search.failed);
    assert!(search.prompt_label().contains("reverse-i-search"));
}

#[test]
fn isearch_again_walks_older_matches() {
    let mut history = History::default();
    history.push("foo one");
    history.push("bar");
    history.push("foo two");

    let mut search = HistoryISearch::start(&history, "");
    search.push_char('f');
    search.push_char('o');
    search.push_char('o');
    assert_eq!(search.display_line(), "foo two");
    search.again();
    assert_eq!(search.display_line(), "foo one");
}

#[test]
fn isearch_failed_keeps_draft_and_backspace_recovers() {
    let mut history = History::default();
    history.push("hello");

    let mut search = HistoryISearch::start(&history, "draft");
    search.push_char('z');
    assert!(search.failed);
    assert_eq!(search.display_line(), "draft");
    assert!(search.prompt_label().contains("failed r-search"));

    search.backspace();
    assert!(!search.failed);
    assert_eq!(search.display_line(), "hello");
    assert_eq!(search.draft(), "draft");
}
