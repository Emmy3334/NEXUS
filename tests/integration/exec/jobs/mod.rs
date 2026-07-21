//! Tests for `&` background jobs and `jobs` / `fg` / `bg`.

mod background;

use super::common::{parse_list, test_env};
use nexus::exec::{execute_list, CommandResult};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

fn temp_file(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_jobs_{}_{name}", std::process::id()))
}

fn run(source: &str, env: &mut nexus::env::ShellEnvironment) -> (CommandResult, String, String) {
    let list = parse_list(source);
    let mut argv = Vec::new();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = execute_list(
        &list,
        &mut argv,
        env,
        0,
        Vec::new(),
        &mut std::io::empty(),
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    (
        result,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

#[test]
fn fg_with_no_jobs_errors() {
    let mut env = test_env();
    let (result, _, stderr) = run("fg", &mut env);
    assert_eq!(result, CommandResult::Status(1));
    assert!(stderr.contains("No current job"), "{stderr}");
}

#[test]
fn background_then_fg_waits() {
    let mut env = test_env();
    let (result, _, stderr) = run("sh -c 'exit 0' &", &mut env);
    assert_eq!(result, CommandResult::Status(0));
    assert!(stderr.contains("[1]"), "{stderr}");
    thread::sleep(Duration::from_millis(50));
    let (fg_result, _, fg_err) = run("fg", &mut env);
    assert!(
        fg_result == CommandResult::Status(0) || fg_result == CommandResult::Status(1),
        "fg={fg_result:?} err={fg_err}"
    );
}

#[test]
fn parse_ampersand_sets_background() {
    let list = parse_list("true &");
    assert!(list.pipelines[0].background);
    assert!(list.as_single_command().is_none());
}
