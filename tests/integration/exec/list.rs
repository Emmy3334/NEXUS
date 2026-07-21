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
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(result, CommandResult::Exit(7));
}
