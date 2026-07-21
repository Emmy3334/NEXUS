//! Background external, builtin, subshell, and pipeline coverage.

use super::{run, temp_file};
use crate::exec::common::test_env;
use nexus::exec::CommandResult;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

#[test]
fn external_registers_job() {
    let path = temp_file("bg_out.txt");
    let _ = fs::remove_file(&path);
    let mut env = test_env();
    let source = format!("sh -c 'echo hi > {}' &", path.display());
    let (result, _, stderr) = run(&source, &mut env);
    assert_eq!(result, CommandResult::Status(0));
    assert!(stderr.contains("[1]"), "stderr={stderr}");
    wait_for_contents(&path, "hi\n");
    let (jobs_result, _, _) = run("jobs", &mut env);
    assert_eq!(jobs_result, CommandResult::Status(0));
    let _ = fs::remove_file(path);
}

#[test]
fn builtin_runs_in_isolated_shell() {
    let mut env = test_env();
    let (result, _, stderr) = run("cd /tmp &", &mut env);
    assert_eq!(result, CommandResult::Status(0));
    assert!(stderr.contains("[1]"), "{stderr}");
}

#[test]
fn subshell_runs() {
    let path = temp_file("subshell.txt");
    let _ = fs::remove_file(&path);
    let mut env = test_env();
    let source = format!("( echo subshell > {} ) &", path.display());
    let (result, _, stderr) = run(&source, &mut env);
    assert_eq!(result, CommandResult::Status(0));
    assert!(stderr.contains("[1]"), "{stderr}");
    wait_for_contents(&path, "subshell\n");
    let _ = fs::remove_file(path);
}

#[test]
fn pipeline_runs() {
    let path = temp_file("pipeline.txt");
    let _ = fs::remove_file(&path);
    let mut env = test_env();
    let source = format!("printf hello | cat > {} &", path.display());
    let (result, _, stderr) = run(&source, &mut env);
    assert_eq!(result, CommandResult::Status(0));
    assert!(stderr.contains("[1]"), "{stderr}");
    wait_for_contents(&path, "hello");
    let _ = fs::remove_file(path);
}

fn wait_for_contents(path: &Path, expected: &str) {
    for _ in 0..50 {
        if fs::read_to_string(path).ok().as_deref() == Some(expected) {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(fs::read_to_string(path).unwrap(), expected);
}
