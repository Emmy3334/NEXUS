//! Precise tests for every `history` builtin flag.

use nexus::builtins::{self, BuiltinResult};
use nexus::env::ShellEnvironment;
use nexus::history::History;
use nexus::repl;
use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::time::{Duration, UNIX_EPOCH};

fn empty_env() -> ShellEnvironment {
    ShellEnvironment::from_map(BTreeMap::new())
}

fn seeded(lines: &[&str]) -> ShellEnvironment {
    let mut env = empty_env();
    for line in lines {
        env.history.push(*line);
    }
    env
}

fn status_of(result: Option<BuiltinResult>) -> u8 {
    match result {
        Some(BuiltinResult::Status(code)) => code,
        other => panic!("expected Status, got {other:?}"),
    }
}

fn run_history(env: &mut ShellEnvironment, args: &[&str]) -> (u8, String, String) {
    let argv: Vec<String> = std::iter::once("history".to_string())
        .chain(args.iter().map(|s| (*s).to_string()))
        .collect();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = status_of(builtins::try_run(&argv, env, 0, &mut stdout, &mut stderr).unwrap());
    (
        code,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_histflag_{name}_{}", std::process::id()))
}

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

#[test]
fn flag_default_prints_numbered_oldest_first() {
    let mut env = seeded(&["one", "two", "three"]);
    let (code, out, err) = run_history(&mut env, &[]);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(out, "    1  one\n    2  two\n    3  three\n");
}

#[test]
fn flag_h_hides_event_numbers() {
    let mut env = seeded(&["one", "two"]);
    let (code, out, err) = run_history(&mut env, &["-h"]);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(out, "one\ntwo\n");
    assert!(!out.contains("1"), "out={out}");
}

#[test]
fn flag_r_prints_newest_first() {
    let mut env = seeded(&["one", "two", "three"]);
    let (code, out, err) = run_history(&mut env, &["-r"]);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(out, "    3  three\n    2  two\n    1  one\n");
}

#[test]
fn flag_n_prints_only_last_n() {
    let mut env = seeded(&["one", "two", "three", "four"]);
    let (code, out, err) = run_history(&mut env, &["2"]);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(out, "    3  three\n    4  four\n");
}

#[test]
fn flag_r_with_n_reverses_the_window() {
    let mut env = seeded(&["one", "two", "three", "four"]);
    let (code, out, err) = run_history(&mut env, &["-r", "2"]);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(out, "    4  four\n    3  three\n");
}

#[test]
fn flag_h_and_r_combine() {
    let mut env = seeded(&["one", "two", "three"]);
    let (code, out, err) = run_history(&mut env, &["-hr"]);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(out, "three\ntwo\none\n");
}

#[test]
fn flag_t_emits_unix_timestamp_comments() {
    let mut env = empty_env();
    env.history
        .push_at("stamped", UNIX_EPOCH + Duration::from_secs(1_700_000_000));
    let (code, out, err) = run_history(&mut env, &["-T"]);
    assert_eq!(code, 0, "err={err}");
    assert!(out.starts_with("#1700000000\n"), "out={out}");
    assert!(out.contains("    1  stamped\n"), "out={out}");
}

#[test]
fn flag_th_timestamps_without_numbers() {
    let mut env = empty_env();
    env.history
        .push_at("line", UNIX_EPOCH + Duration::from_secs(42));
    let (code, out, err) = run_history(&mut env, &["-Th"]);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(out, "#42\nline\n");
}

#[test]
fn flag_c_clears_all_events() {
    let mut env = seeded(&["one", "two"]);
    let (code, out, err) = run_history(&mut env, &["-c"]);
    assert_eq!(code, 0, "err={err}");
    assert!(out.is_empty(), "out={out}");
    assert!(env.history.is_empty());
    let (code, out, err) = run_history(&mut env, &[]);
    assert_eq!(code, 0, "err={err}");
    assert!(out.is_empty(), "out={out}");
}

#[test]
fn flag_s_writes_histfile_with_timestamps() {
    let dir = scratch("s");
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("hist");
    let mut env = empty_env();
    env.history
        .push_at("alpha", UNIX_EPOCH + Duration::from_secs(10));
    env.history
        .push_at("beta", UNIX_EPOCH + Duration::from_secs(20));
    let path = file.to_string_lossy().into_owned();
    let (code, _, err) = run_history(&mut env, &["-S", &path]);
    assert_eq!(code, 0, "err={err}");
    let body = fs::read_to_string(&file).unwrap();
    assert_eq!(body, "#10\nalpha\n#20\nbeta\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn flag_l_appends_histfile_into_memory() {
    let dir = scratch("l");
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("hist");
    let mut foreign = History::default();
    foreign.push_at("fromfile", UNIX_EPOCH + Duration::from_secs(5));
    foreign.save_to(&file).unwrap();

    let mut env = seeded(&["existing"]);
    let path = file.to_string_lossy().into_owned();
    let (code, _, err) = run_history(&mut env, &["-L", &path]);
    assert_eq!(code, 0, "err={err}");
    let lines: Vec<_> = env.history.iter().map(|(_, e)| e.line.as_str()).collect();
    assert_eq!(lines, ["existing", "fromfile"]);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn flag_m_merges_by_timestamp() {
    let dir = scratch("m");
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("hist");
    let mut foreign = History::default();
    foreign.push_at("older", UNIX_EPOCH + Duration::from_secs(1));
    foreign.push_at("newer", UNIX_EPOCH + Duration::from_secs(100));
    foreign.save_to(&file).unwrap();

    let mut env = empty_env();
    env.history
        .push_at("middle", UNIX_EPOCH + Duration::from_secs(50));
    let path = file.to_string_lossy().into_owned();
    let (code, _, err) = run_history(&mut env, &["-M", &path]);
    assert_eq!(code, 0, "err={err}");
    let lines: Vec<_> = env.history.iter().map(|(_, e)| e.line.as_str()).collect();
    assert_eq!(lines, ["older", "middle", "newer"]);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn flag_s_uses_histfile_local_when_no_path() {
    let dir = scratch("default");
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("nexus.hist");
    let mut env = seeded(&["saved"]);
    env.set_local("histfile", file.to_string_lossy().as_ref());
    let (code, _, err) = run_history(&mut env, &["-S"]);
    assert_eq!(code, 0, "err={err}");
    assert!(file.exists(), "histfile should be created");
    let body = fs::read_to_string(&file).unwrap();
    assert!(body.contains("saved"), "body={body}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn flag_unknown_option_errors() {
    let mut env = seeded(&["one"]);
    let (code, out, err) = run_history(&mut env, &["-z"]);
    assert_eq!(code, 1);
    assert!(err.contains("Unknown option"), "err={err}");
    assert!(out.is_empty());
}

#[test]
fn flag_c_rejects_extra_args() {
    let mut env = seeded(&["one"]);
    let (code, _, err) = run_history(&mut env, &["-c", "x"]);
    assert_eq!(code, 1);
    assert!(err.contains("Too many arguments"), "err={err}");
}

#[test]
fn flag_bad_number_errors() {
    let mut env = seeded(&["one"]);
    let (code, _, err) = run_history(&mut env, &["nope"]);
    assert_eq!(code, 1);
    assert!(err.contains("Badly formed number"), "err={err}");
}

#[test]
fn flags_work_in_repl_script() {
    let dir = scratch("repl");
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("hist");
    let script = format!(
        "true\nfalse\nhistory -h\nhistory -r 1\nhistory -T 1\nhistory -S {0}\nhistory -c\nhistory -L {0}\nhistory -h\n",
        file.display()
    );
    let (code, out, err) = run_script(&script);
    assert_eq!(code, 0, "err={err}");
    assert!(out.contains("true\n"), "out={out}");
    assert!(out.contains("false\n"), "out={out}");
    assert!(out.contains('#'), "expected -T timestamp, out={out}");
    // After -L, saved events are back (plus the history commands that followed load).
    assert!(
        out.lines().any(|l| l == "true" || l.ends_with("true")),
        "out={out}"
    );
    let _ = fs::remove_dir_all(&dir);
}
