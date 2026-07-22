//! Extra coverage for history event / word designators and modifiers.

use nexus::history::{self, History};

fn expand(raw: &str, history: &mut History) -> Result<String, history::HistoryError> {
    let mut out = String::new();
    history::expand_line(raw, history, &mut out)?;
    Ok(out)
}

#[test]
fn event_braces_and_current_line() {
    let mut h = History::default();
    h.push("vi wumpus.man");
    assert_eq!(expand("!{v}doc", &mut h).unwrap(), "vi wumpus.mandoc");
    assert_eq!(expand("echo !#:0", &mut h).unwrap(), "echo echo");
}

#[test]
fn word_ranges_and_omit_last() {
    let mut h = History::default();
    h.push("a b c d");
    assert_eq!(expand("!!:0-1", &mut h).unwrap(), "a b");
    assert_eq!(expand("!!:-2", &mut h).unwrap(), "a b c");
    assert_eq!(expand("!!:1-", &mut h).unwrap(), "b c");
    assert_eq!(expand("!!:2*", &mut h).unwrap(), "c d");
    assert_eq!(expand("!!:-", &mut h).unwrap(), "a b c");
}

#[test]
fn search_percent_designator() {
    let mut h = History::default();
    h.push("diff old.man new.man");
    assert_eq!(expand("!?old?", &mut h).unwrap(), "diff old.man new.man");
    assert_eq!(expand("!%", &mut h).unwrap(), "old.man");
}

#[test]
fn modifiers_case_quote_global_and_repeat() {
    let mut h = History::default();
    h.push("echo hello world");
    assert_eq!(expand("!!:1:u", &mut h).unwrap(), "Hello");
    assert_eq!(expand("!!*:gu", &mut h).unwrap(), "Hello World");
    h.push("echo aaa");
    assert_eq!(expand("!!:1:as/a/b/", &mut h).unwrap(), "bbb");
    h.push("echo hi");
    assert_eq!(expand("!!:1:q", &mut h).unwrap(), "'hi'");
}

#[test]
fn modifier_ampersand_repeats_last_subst() {
    let mut h = History::default();
    h.push("echo foo bar foo");
    assert_eq!(expand("!:s/foo/FOO/", &mut h).unwrap(), "echo FOO bar foo");
    h.push("echo foo bar foo");
    assert_eq!(expand("!:&", &mut h).unwrap(), "echo FOO bar foo");
}

#[test]
fn literal_bang_when_followed_by_space_or_equals() {
    let mut h = History::default();
    h.push("echo x");
    assert_eq!(expand("echo ! =", &mut h).unwrap(), "echo ! =");
    assert_eq!(expand("echo !(", &mut h).unwrap(), "echo !(");
}

#[test]
fn numeric_prefix_is_not_event_number() {
    let mut h = History::default();
    h.push("3dplot data");
    assert_eq!(expand("!3d", &mut h).unwrap(), "3dplot data");
}
