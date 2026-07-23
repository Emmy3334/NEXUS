//! Native `@docker` builtin tests (soft-skip when daemon down).

mod docker_errors;
mod docker_run;

use docker_run::run_docker;
use nexus::builtins::BuiltinResult;
use nexus::heal;
use nexus::repl::complete;
use std::process::Command;

#[test]
fn docker_usage_and_unknown() {
    let (result, _, err) = run_docker(&[]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("usage: @docker"));

    let (result, _, err) = run_docker(&["help"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("usage:"));

    let (result, _, err) = run_docker(&["images"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("unknown"));
}

#[test]
fn docker_ps_soft_or_header() {
    if heal::daemon_reachable() {
        let (result, out, err) = run_docker(&["ps"]);
        assert_eq!(result, BuiltinResult::Status(0), "stderr={err}");
        assert!(out.contains("CONTAINER ID") && out.contains("NAMES"));
        return;
    }
    let (result, _, err) = run_docker(&["ps"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(!err.is_empty());
}

#[test]
fn docker_logs_requires_target() {
    let (result, _, err) = run_docker(&["logs"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("usage:"));
}

#[test]
fn docker_complete_subcommands() {
    let mut buffer = "@docker ".to_owned();
    let mut cursor = buffer.len();
    let matches = complete(&mut buffer, &mut cursor);
    assert!(matches.iter().any(|m| m == "ps"));
    assert!(matches.iter().any(|m| m == "logs"));
}

#[test]
fn docker_logs_live_when_daemon_up() {
    if !Command::new("docker")
        .args(["info"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return;
    }
    let name = format!("nexus_docker_ps_{}", std::process::id());
    let status = Command::new("docker")
        .args([
            "run",
            "-d",
            "--rm",
            "--name",
            &name,
            "alpine:3.20",
            "sh",
            "-c",
            "echo hello-nexus; sleep 30",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    if !status.map(|s| s.success()).unwrap_or(false) {
        return;
    }

    let (ps_result, ps_out, ps_err) = run_docker(&["ps"]);
    let (log_result, log_out, log_err) = run_docker(&["logs", &name]);
    let _ = Command::new("docker")
        .args(["rm", "-f", &name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    assert_eq!(ps_result, BuiltinResult::Status(0), "stderr={ps_err}");
    assert!(ps_out.contains(&name), "ps_out={ps_out}");
    assert_eq!(log_result, BuiltinResult::Status(0), "stderr={log_err}");
    assert!(
        log_out.contains("hello-nexus"),
        "log_out={log_out} err={log_err}"
    );
}
