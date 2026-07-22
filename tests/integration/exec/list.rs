//! Tests for `;` list execution.

use super::common::{parse_list, test_env};
use nexus::exec::{execute_list, CommandResult};

#[test]
fn semicolon_list_runs_in_sequence() {
    let source = "false ; true";
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
    assert_eq!(result, CommandResult::Status(0));
    assert!(stderr.is_empty());
}

#[test]
fn semicolon_list_keeps_last_status() {
    let source = "true ; false";
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
    assert_eq!(result, CommandResult::Status(1));
}

#[test]
fn exit_in_list_stops_remaining_commands() {
    let source = "exit 7 ; false";
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
    assert_eq!(result, CommandResult::Exit(7));
}

#[test]
fn and_skips_right_when_left_fails() {
    let source = "false && true";
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
    assert_eq!(result, CommandResult::Status(1));
}

#[test]
fn or_skips_right_when_left_succeeds() {
    let source = "true || false";
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
    assert_eq!(result, CommandResult::Status(0));
}

#[test]
fn and_or_chain_short_circuits() {
    let source = "false && true || true";
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
    assert_eq!(result, CommandResult::Status(0));
}

#[test]
fn skipped_and_pipeline_discards_heredoc_body() {
    use std::fs;
    use std::path::PathBuf;

    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("and_or_heredoc.txt");
    let _ = fs::remove_file(&path);
    let source = format!("false && cat <<EOF ; cat <<EOF > {}", path.display());
    let list = parse_list(&source);
    let mut env = test_env();
    let mut argv = Vec::new();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let bodies = vec!["skipped\n".to_owned(), "kept\n".to_owned()];
    let result = execute_list(
        &list,
        &mut argv,
        &mut env,
        0,
        bodies,
        &mut std::io::empty(),
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(result, CommandResult::Status(0));
    assert_eq!(fs::read_to_string(&path).unwrap(), "kept\n");
    let _ = fs::remove_file(&path);
}
