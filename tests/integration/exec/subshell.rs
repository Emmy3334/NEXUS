//! Tests for `( … )` subshell grouping.

use super::common::{parse_list, test_env};
use nexus::exec::{collect_heredoc_bodies, execute_list, CommandResult};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

fn temp_file(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_subshell_{}_{name}", std::process::id()))
}

fn run(source: &str) -> (CommandResult, String) {
    let list = parse_list(source);
    let mut env = test_env();
    let mut argv = Vec::new();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = execute_list(
        &list,
        &mut argv,
        &mut env,
        0,
        Vec::new(),
        &mut std::io::empty(),
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    (result, String::from_utf8(stderr).unwrap())
}

#[test]
fn subshell_runs_grouped_commands() {
    let path = temp_file("grouped.txt");
    let _ = fs::remove_file(&path);
    let source = format!("(echo one; echo two) > {}", path.display());
    let (result, stderr) = run(&source);
    assert_eq!(result, CommandResult::Status(0));
    assert!(stderr.is_empty());
    assert_eq!(fs::read_to_string(&path).unwrap(), "one\ntwo\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn subshell_env_mutations_do_not_leak() {
    let list = parse_list("(setenv NEXUS_SUB_MARK child)");
    let mut env = test_env();
    env.set("NEXUS_SUB_MARK", "parent");
    let mut argv = Vec::new();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = execute_list(
        &list,
        &mut argv,
        &mut env,
        0,
        Vec::new(),
        &mut std::io::empty(),
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(result, CommandResult::Status(0));
    assert_eq!(env.get("NEXUS_SUB_MARK"), Some("parent"));
    assert!(stderr.is_empty());
}

#[test]
fn subshell_exit_does_not_exit_parent() {
    let path = temp_file("after_exit.txt");
    let _ = fs::remove_file(&path);
    let source = format!("(exit 3); echo still > {}", path.display());
    let (result, stderr) = run(&source);
    assert_eq!(result, CommandResult::Status(0));
    assert!(stderr.is_empty());
    assert_eq!(fs::read_to_string(&path).unwrap(), "still\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn subshell_group_redirect_writes_file() {
    let path = temp_file("group_out.txt");
    let _ = fs::remove_file(&path);
    let source = format!("(echo grouped) > {}", path.display());
    let (result, stderr) = run(&source);
    assert_eq!(result, CommandResult::Status(0));
    assert!(stderr.is_empty());
    assert_eq!(fs::read_to_string(&path).unwrap(), "grouped\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn subshell_in_pipeline() {
    let path = temp_file("pipe.txt");
    let _ = fs::remove_file(&path);
    let source = format!("(echo pipe-me) | cat > {}", path.display());
    let (result, stderr) = run(&source);
    assert_eq!(result, CommandResult::Status(0));
    assert!(stderr.is_empty());
    assert_eq!(fs::read_to_string(&path).unwrap(), "pipe-me\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn nested_subshell() {
    let path = temp_file("nest.txt");
    let _ = fs::remove_file(&path);
    let source = format!("((echo nest)) > {}", path.display());
    let (result, stderr) = run(&source);
    assert_eq!(result, CommandResult::Status(0));
    assert!(stderr.is_empty());
    assert_eq!(fs::read_to_string(&path).unwrap(), "nest\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn subshell_heredoc_on_inner_command() {
    let path = temp_file("heredoc.txt");
    let _ = fs::remove_file(&path);
    let source = format!("(cat << EOF) > {}", path.display());
    let list = parse_list(&source);
    let mut env = test_env();
    let mut argv = Vec::new();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut input = Cursor::new("hello\nEOF\n");
    let bodies = collect_heredoc_bodies(&list, &mut env, 0, &mut input, &mut stderr)
        .unwrap()
        .unwrap();
    let result = execute_list(
        &list,
        &mut argv,
        &mut env,
        0,
        bodies,
        &mut input,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(result, CommandResult::Status(0));
    assert!(stderr.is_empty());
    assert_eq!(fs::read_to_string(&path).unwrap(), "hello\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn subshell_status_is_last_inner_command() {
    let (result, stderr) = run("(true; false)");
    assert_eq!(result, CommandResult::Status(1));
    assert!(stderr.is_empty());
}
