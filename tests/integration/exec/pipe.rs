//! Tests for `|` pipeline execution.

use super::common::{parse_list, test_env};
use nexus::exec::{execute_list, CommandResult};

#[test]
fn pipe_uses_last_command_status() {
    let source = "true | false";
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

    let source = "false | true";
    let list = parse_list(source);
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
fn multipipe_status_is_last_stage() {
    let source = "true | true | false";
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
fn exit_in_pipeline_does_not_kill_shell() {
    let source = "exit 9 | true";
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
fn builtin_env_can_feed_pipe() {
    let source = "env | true";
    let list = parse_list(source);
    let mut env = test_env();
    env.set("NEXUS_PIPE_TEST", "1");
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
fn semicolon_then_pipe() {
    let source = "false ; true | false";
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
fn missing_left_stage_still_runs_right() {
    let source = "nexus_no_such_command_pipe_42 | true";
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
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("Command not found"));
    assert!(err.contains("nexus_no_such_command_pipe_42"));
}

#[test]
fn missing_right_stage_status_is_127() {
    let source = "true | nexus_no_such_command_pipe_99";
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
    assert_eq!(result, CommandResult::Status(127));
    assert!(String::from_utf8(stderr)
        .unwrap()
        .contains("Command not found"));
}
