//! Tests for the `bindkey` builtin.

use nexus::builtins::{self, BuiltinResult};
use nexus::env::ShellEnvironment;
use nexus::keybind::{Action, Binding};
use nexus::repl;
use std::collections::BTreeMap;
use std::io::Cursor;

fn empty_env() -> ShellEnvironment {
    ShellEnvironment::from_map(BTreeMap::new())
}

fn status_of(result: Option<BuiltinResult>) -> u8 {
    match result {
        Some(BuiltinResult::Status(code)) => code,
        other => panic!("expected Status, got {other:?}"),
    }
}

fn run(argv: &[&str], env: &mut ShellEnvironment) -> (u8, String, String) {
    let argv: Vec<String> = argv.iter().map(|s| (*s).to_owned()).collect();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = status_of(builtins::try_run(&argv, env, 0, &mut stdout, &mut stderr).unwrap());
    (
        code,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

#[test]
fn bindkey_lists_defaults() {
    let mut env = empty_env();
    let (code, out, err) = run(&["bindkey"], &mut env);
    assert_eq!(code, 0);
    assert!(err.is_empty());
    assert!(out.contains("tty-sigintr"));
    assert!(out.contains("complete-word"));
}

#[test]
fn bindkey_binds_and_shows_one() {
    let mut env = empty_env();
    let (code, _, err) = run(&["bindkey", "^X", "tty-sigintr"], &mut env);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(
        env.key_bindings.lookup_in(&[0x18], false),
        Some(&Binding::Action(Action::Interrupt))
    );
    let (code, out, err) = run(&["bindkey", "^X"], &mut env);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(out.trim(), "tty-sigintr");
}

#[test]
fn bindkey_arrow_and_reset() {
    let mut env = empty_env();
    let (code, _, err) = run(&["bindkey", "-k", "up", "complete-word"], &mut env);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(
        env.key_bindings.lookup_in(b"\x1b[A", false),
        Some(&Binding::Action(Action::Complete))
    );
    let (code, _, err) = run(&["bindkey", "-d"], &mut env);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(
        env.key_bindings.lookup_in(b"\x1b[A", false),
        Some(&Binding::Action(Action::HistoryUp))
    );
}

#[test]
fn bindkey_remove_and_list_commands() {
    let mut env = empty_env();
    let (code, _, err) = run(&["bindkey", "-r", "^C"], &mut env);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(env.key_bindings.lookup_in(&[0x03], false), None);
    let (code, out, err) = run(&["bindkey", "-l"], &mut env);
    assert_eq!(code, 0, "err={err}");
    assert!(out.contains("accept-line"));
    assert!(out.contains("end-of-file"));
}

#[test]
fn bindkey_persists_across_repl_commands() {
    let (code, out, err) = {
        let mut stdin = Cursor::new("bindkey ^X tty-sigintr\nbindkey ^X\n");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
        (
            code,
            String::from_utf8(stdout).unwrap(),
            String::from_utf8(stderr).unwrap(),
        )
    };
    assert_eq!(code, 0, "err={err}");
    assert!(out.contains("tty-sigintr"), "out={out}");
}

#[test]
fn bindkey_usage_emacs_vi_and_b_flag() {
    let mut env = empty_env();
    let (code, out, err) = run(&["bindkey", "-u"], &mut env);
    assert_eq!(code, 0, "err={err}");
    assert!(out.contains("Usage:"));
    let (code, _, err) = run(&["bindkey", "-b", "C-x", "tty-sigintr"], &mut env);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(
        env.key_bindings.lookup_in(&[0x18], false),
        Some(&Binding::Action(Action::Interrupt))
    );
    let (code, _, err) = run(&["bindkey", "-v"], &mut env);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(
        env.key_bindings.lookup_in(b"\x1b", false),
        Some(&Binding::Action(Action::ViCmdMode))
    );
    let (code, _, err) = run(&["bindkey", "-a", "h"], &mut env);
    assert_eq!(code, 0, "err={err}");
    let (code, out, err) = run(&["bindkey", "-a", "h"], &mut env);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(out.trim(), "backward-char");
    let (code, _, err) = run(&["bindkey", "-e"], &mut env);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(env.key_bindings.lookup_in(b"\x1b", false), None);
}

#[test]
fn bindkey_command_literal_and_endopts() {
    let mut env = empty_env();
    let (code, _, err) = run(&["bindkey", "-c", "^L", "clear"], &mut env);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(
        env.key_bindings.lookup_in(&[0x0c], false),
        Some(&Binding::Command("clear".into()))
    );
    let (code, _, err) = run(&["bindkey", "-s", "--", "-x", "hello"], &mut env);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(
        env.key_bindings.lookup_in(b"-x", false),
        Some(&Binding::Literal("hello".into()))
    );
}
