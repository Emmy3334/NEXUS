//! Tests for `history` builtin and `!` event expansion.

use nexus::history::{self, History};
use nexus::repl;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

fn run_script(input: &str) -> (u8, String, String) {
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    (
        code,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_hist_{name}_{}", std::process::id()))
}

fn expand(raw: &str, history: &mut History) -> Result<String, history::HistoryError> {
    let mut out = String::new();
    history::expand_line(raw, history, &mut out)?;
    Ok(out)
}

#[test]
fn expand_bang_bang_and_absolute() {
    let mut h = History::default();
    h.push("echo one");
    h.push("echo two");
    assert_eq!(expand("!!", &mut h).unwrap(), "echo two");
    assert_eq!(expand("!1", &mut h).unwrap(), "echo one");
    assert_eq!(expand("!-2", &mut h).unwrap(), "echo one");
}

#[test]
fn expand_prefix_and_quotes() {
    let mut h = History::default();
    h.push("echo hello");
    h.push("printf hi");
    assert_eq!(expand("!ec", &mut h).unwrap(), "echo hello");
    assert_eq!(expand("echo '!!'", &mut h).unwrap(), "echo '!!'");
    assert_eq!(expand(r#"echo "!!""#, &mut h).unwrap(), r#"echo "!!""#);
}

#[test]
fn expand_event_not_found() {
    let mut h = History::default();
    assert_eq!(
        expand("!!", &mut h).unwrap_err().message(),
        "Event not found."
    );
}

#[test]
fn expand_word_designators() {
    let mut h = History::default();
    h.push("cmd aa bb cc");
    assert_eq!(expand("!!:0", &mut h).unwrap(), "cmd");
    assert_eq!(expand("!!$", &mut h).unwrap(), "cc");
    assert_eq!(expand("!!*", &mut h).unwrap(), "aa bb cc");
    assert_eq!(expand("!!:1-2", &mut h).unwrap(), "aa bb");
    assert_eq!(expand("!!^", &mut h).unwrap(), "aa");
}

#[test]
fn expand_search_and_modifiers() {
    let mut h = History::default();
    h.push("cp /tmp/foo.bar /tmp/out");
    assert_eq!(
        expand("!?foo?", &mut h).unwrap(),
        "cp /tmp/foo.bar /tmp/out"
    );
    assert_eq!(expand("!!:$:h", &mut h).unwrap(), "/tmp");
    assert_eq!(expand("!!:$:t", &mut h).unwrap(), "out");
    h.push("echo /tmp/foo.bar");
    assert_eq!(expand("!!:1:r", &mut h).unwrap(), "/tmp/foo");
    assert_eq!(expand("!!:1:e", &mut h).unwrap(), "bar");
}

#[test]
fn expand_subst_and_quick() {
    let mut h = History::default();
    h.push("echo hello world");
    assert_eq!(expand("!:s/hello/hi/", &mut h).unwrap(), "echo hi world");
    h.push("echo hello world");
    assert_eq!(expand("^hello^hi", &mut h).unwrap(), "echo hi world");
}

#[test]
fn expand_print_only() {
    let mut h = History::default();
    h.push("echo hi");
    let mut out = String::new();
    let outcome = history::expand_line("!!:p", &mut h, &mut out).unwrap();
    assert_eq!(out, "echo hi");
    assert!(outcome.print_only);
}

#[test]
fn history_builtin_lists_events() {
    let (code, out, err) = run_script("true\nfalse\nhistory\n");
    assert_eq!(code, 0, "err={err}");
    assert!(out.contains("1  true"), "out={out}");
    assert!(out.contains("2  false"), "out={out}");
    assert!(out.contains("3  history"), "out={out}");
}

#[test]
fn bang_bang_reexecutes_last_command() {
    let dir = scratch("bang");
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("o.txt");
    let script = format!(
        "printf '%s\\n' hi > {0}\n\
         !!\n",
        out.display()
    );
    let (code, _, err) = run_script(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "hi\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn bang_n_and_prefix_in_repl() {
    let dir = scratch("num");
    fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a.txt");
    let b = dir.join("b.txt");
    let script = format!(
        "printf '%s\\n' one > {0}\n\
         printf '%s\\n' two > {1}\n\
         !1\n\
         !pr\n",
        a.display(),
        b.display()
    );
    let (code, _, err) = run_script(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&a).unwrap(), "one\n");
    assert_eq!(fs::read_to_string(&b).unwrap(), "two\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn bang_relative_reexecutes() {
    let dir = scratch("rel");
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("o.txt");
    let script = format!(
        "printf '%s\\n' first > {0}\n\
         printf '%s\\n' second > {0}\n\
         !-2\n",
        out.display()
    );
    let (code, _, err) = run_script(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "first\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn bang_missing_event_reports_error() {
    let (code, out, err) = run_script("!!\n");
    assert_eq!(code, 1);
    assert!(err.contains("Event not found."), "err={err}");
    assert!(out.is_empty(), "out={out}");
}

#[test]
fn quoted_bang_is_literal() {
    let dir = scratch("quote");
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("o.txt");
    let script = format!("printf '%s\\n' '!!' > {}\n", out.display());
    let (code, _, err) = run_script(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "!!\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn bang_print_only_does_not_run() {
    let dir = scratch("ponly");
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("o.txt");
    let script = format!(
        "printf '%s\\n' hi > {0}\n\
         !!:p\n",
        out.display()
    );
    let (code, stdout, err) = run_script(&script);
    assert_eq!(code, 0, "err={err}");
    assert!(stdout.contains("printf"), "stdout={stdout}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "hi\n");
    let _ = fs::remove_dir_all(&dir);
}
