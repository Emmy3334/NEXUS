//! Tests for `alias` / `unalias` and command-word expansion.

use nexus::builtins::{self, BuiltinResult};
use nexus::env::ShellEnvironment;
use nexus::repl;
use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

fn empty_env() -> ShellEnvironment {
    ShellEnvironment::from_map(BTreeMap::new())
}

fn status_of(result: Option<BuiltinResult>) -> u8 {
    match result {
        Some(BuiltinResult::Status(code)) => code,
        other => panic!("expected Status, got {other:?}"),
    }
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

fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_alias_{name}_{}", std::process::id()))
}

#[test]
fn alias_define_list_and_lookup() {
    let mut env = empty_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        status_of(
            builtins::try_run(
                &["alias".into(), "ll".into(), "ls".into(), "-la".into()],
                &mut env,
                0,
                &mut stdout,
                &mut stderr,
            )
            .unwrap()
        ),
        0
    );
    assert_eq!(env.alias_get("ll"), Some("ls -la"));
    stdout.clear();
    assert_eq!(
        status_of(
            builtins::try_run(&["alias".into()], &mut env, 0, &mut stdout, &mut stderr).unwrap()
        ),
        0
    );
    assert_eq!(String::from_utf8(stdout).unwrap(), "ll\tls -la\n");
}

#[test]
fn unalias_removes() {
    let mut env = empty_env();
    env.alias_set("ll", "ls -la");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        status_of(
            builtins::try_run(
                &["unalias".into(), "ll".into()],
                &mut env,
                0,
                &mut stdout,
                &mut stderr,
            )
            .unwrap()
        ),
        0
    );
    assert_eq!(env.alias_get("ll"), None);
}

#[test]
fn alias_expands_command_word() {
    let dir = scratch("exp");
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("o.txt");
    let script = format!(
        "alias say printf\n\
         say '%s\\n' hi > {}\n",
        out.display()
    );
    let (code, _, err) = run_script(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "hi\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn alias_appends_extra_args() {
    let dir = scratch("args");
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("o.txt");
    let script = format!(
        "alias greet printf\n\
         greet '%s\\n' hello > {}\n",
        out.display()
    );
    let (code, _, err) = run_script(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "hello\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn recursive_alias_does_not_loop() {
    let (code, _, err) = run_script("alias a b\nalias b a\na\n");
    // Ends as missing command `a` or `b` with 127, not hang.
    assert_eq!(code, 127, "err={err}");
    assert!(err.contains("Command not found"));
}

#[test]
fn alias_and_unalias_not_self_expanded() {
    let (code, out, err) = run_script("alias alias printf\nalias\n");
    // `alias` builtin must still list (or be empty), not run printf.
    assert_eq!(code, 0, "err={err}");
    assert!(err.is_empty());
    assert!(out.is_empty() || out.contains('\t'));
}

#[test]
fn alias_in_pipeline_stage() {
    let dir = scratch("pipe");
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("o.txt");
    let script = format!(
        "alias p printf\n\
         p '%s\\n' piped | cat > {}\n",
        out.display()
    );
    let (code, _, err) = run_script(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "piped\n");
    let _ = fs::remove_dir_all(&dir);
}
