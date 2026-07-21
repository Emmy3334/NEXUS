//! Integration tests for the REPL.

use nexus::repl;
use std::io::Cursor;

const PROMPT: &str = "$> ";

fn run_piped(input: &str) -> (u8, Vec<u8>, Vec<u8>) {
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    (code, stdout, stderr)
}

#[test]
fn eof_exits_zero_without_prompt_noise() {
    let (code, stdout, stderr) = run_piped("");
    assert_eq!(code, 0);
    assert!(stdout.is_empty());
    assert!(stderr.is_empty());
}

#[test]
fn blank_lines_then_true_exits_zero() {
    let (code, stdout, stderr) = run_piped("\n   \ntrue\n");
    assert_eq!(code, 0);
    assert!(stdout.is_empty());
    assert!(stderr.is_empty());
}

#[test]
fn false_propagates_exit_status() {
    let (code, _, stderr) = run_piped("false\n");
    assert_eq!(code, 1);
    assert!(stderr.is_empty());
}

#[test]
fn missing_command_writes_stderr_and_returns_127() {
    let (code, _, stderr) = run_piped("nexus_no_such_command_42\n");
    assert_eq!(code, 127);
    let message = String::from_utf8(stderr).unwrap();
    assert!(message.contains("Command not found"));
}

#[test]
fn last_status_wins_across_commands() {
    let (code, _, _) = run_piped("true\nfalse\n");
    assert_eq!(code, 1);
    let (code, _, _) = run_piped("false\ntrue\n");
    assert_eq!(code, 0);
}

#[test]
fn exit_builtin_ends_repl() {
    let (code, _, stderr) = run_piped("false\nexit\ntrue\n");
    assert_eq!(code, 1);
    assert!(stderr.is_empty());
}

#[test]
fn exit_with_explicit_status() {
    let (code, _, _) = run_piped("exit 42\n");
    assert_eq!(code, 42);
}

#[test]
fn interactive_prints_prompt_before_each_read() {
    let mut stdin = Cursor::new("\ntrue\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, true).unwrap();
    assert_eq!(code, 0);
    let out = String::from_utf8(stdout).unwrap();
    assert_eq!(out.matches(PROMPT).count(), 3);
}

#[test]
fn semicolon_lists_execute_in_order() {
    let (code, _, stderr) = run_piped("false ; true\n");
    assert_eq!(code, 0);
    assert!(stderr.is_empty());

    let (code, _, stderr) = run_piped("true ; false\n");
    assert_eq!(code, 1);
    assert!(stderr.is_empty());
}

#[test]
fn pipes_execute_and_use_last_status() {
    let (code, _, stderr) = run_piped("true | false\n");
    assert_eq!(code, 1);
    assert!(stderr.is_empty());

    let (code, _, stderr) = run_piped("false | true\n");
    assert_eq!(code, 0);
    assert!(stderr.is_empty());
}

#[test]
fn exit_in_pipe_does_not_end_shell() {
    let (code, _, stderr) = run_piped("exit 9 | true\nfalse\n");
    assert_eq!(code, 1);
    assert!(stderr.is_empty());
}

#[test]
fn exit_in_semicolon_list_ends_shell() {
    let (code, _, stderr) = run_piped("exit 9 ; false\n");
    assert_eq!(code, 9);
    assert!(stderr.is_empty());
}

#[test]
fn parse_errors_go_to_stderr() {
    let (code, _, stderr) = run_piped("| true\n");
    assert_eq!(code, 1);
    assert!(String::from_utf8(stderr)
        .unwrap()
        .contains("Invalid null command"));

    let (code, _, stderr) = run_piped("cmd << END\n");
    assert_eq!(code, 1);
    assert!(String::from_utf8(stderr)
        .unwrap()
        .contains("heredoc is not implemented"));
}

#[test]
fn redirect_stdout_runs_in_repl() {
    let path = std::env::temp_dir().join(format!("nexus_repl_redir_{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let input = format!("true > {}\n", path.display());
    let (code, _, stderr) = run_piped(&input);
    assert_eq!(code, 0);
    assert!(stderr.is_empty());
    assert!(path.is_file());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn trailing_semicolon_still_runs_simple_command() {
    let (code, _, stderr) = run_piped("true;\n");
    assert_eq!(code, 0);
    assert!(stderr.is_empty());
}

#[test]
fn crlf_line_endings_are_accepted() {
    let (code, _, stderr) = run_piped("true\r\n");
    assert_eq!(code, 0);
    assert!(stderr.is_empty());
}
