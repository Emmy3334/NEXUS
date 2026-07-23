//! `heal` / `doctor` status builtin.

use super::common::test_env;
use nexus::builtins::{self, BuiltinResult};
use nexus::heal::{self, attach_default_backends};
use nexus::kube;

fn run_heal(
    argv: &[&str],
    env: &mut nexus::env::ShellEnvironment,
) -> (BuiltinResult, String, String) {
    let argv: Vec<String> = argv.iter().map(|s| (*s).to_owned()).collect();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = builtins::try_run(&argv, env, 0, &mut stdout, &mut stderr)
        .expect("builtin io")
        .expect("recognized");
    (
        result,
        String::from_utf8(stdout).expect("utf8"),
        String::from_utf8(stderr).expect("utf8"),
    )
}

#[test]
fn heal_status_default_shape() {
    let mut env = test_env();
    attach_default_backends(&mut env);
    let (result, out, err) = run_heal(&["heal"], &mut env);
    assert_eq!(result, BuiltinResult::Status(0));
    assert!(err.is_empty());
    assert!(out.contains("order:"));
    assert!(out.contains("image:"));
    assert!(out.contains("env: none"));
    assert!(out.contains("catch_all: off"));
    assert!(out.contains("quiet: off"));
    assert!(out.contains("session:"));
    assert!(out.contains("wasm: ready"));
}

#[test]
fn heal_status_arg_is_unknown() {
    let mut env = test_env();
    let (result, _, err) = run_heal(&["heal", "status"], &mut env);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("unknown"));
}

#[test]
fn heal_order_wasm_only() {
    let mut env = test_env();
    env.set_local("heal_order", "wasm");
    attach_default_backends(&mut env);
    let (result, out, _) = run_heal(&["heal"], &mut env);
    assert_eq!(result, BuiltinResult::Status(0));
    assert!(out.contains("order: wasm\n") || out.lines().any(|l| l == "order: wasm"));
    assert!(out.contains("wasm: ready"));
    assert!(!out.contains("kube:"));
    assert!(!out.contains("docker:"));
    assert!(out.contains("session: 1 attached"));
}

#[test]
fn heal_quiet_on() {
    let mut env = test_env();
    env.set_local("heal_quiet", "1");
    let (_, out, _) = run_heal(&["heal"], &mut env);
    assert!(out.contains("quiet: on"));
}

#[test]
fn heal_help_usage() {
    let mut env = test_env();
    let (result, _, err) = run_heal(&["heal", "help"], &mut env);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("usage:"));
}

#[test]
fn doctor_alias() {
    let mut env = test_env();
    let (result, out, _) = run_heal(&["doctor"], &mut env);
    assert_eq!(result, BuiltinResult::Status(0));
    assert!(out.contains("order:"));
}

#[test]
fn heal_unknown_subcommand() {
    let mut env = test_env();
    let (result, _, err) = run_heal(&["heal", "widgets"], &mut env);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("unknown"));
}

#[test]
fn heal_probe_lines_match_reachability() {
    let mut env = test_env();
    env.set_local("heal_order", "wasm,kube,docker");
    let (_, out, _) = run_heal(&["heal"], &mut env);
    let kube_line = out.lines().find(|l| l.starts_with("kube:")).expect("kube");
    let docker_line = out
        .lines()
        .find(|l| l.starts_with("docker:"))
        .expect("docker");
    if kube::cluster_reachable() {
        assert!(kube_line.contains("ready"), "{kube_line}");
    } else {
        assert!(kube_line.contains("unavailable"), "{kube_line}");
    }
    if heal::daemon_reachable() {
        assert!(docker_line.contains("ready"), "{docker_line}");
    } else {
        assert!(docker_line.contains("unavailable"), "{docker_line}");
    }
}
