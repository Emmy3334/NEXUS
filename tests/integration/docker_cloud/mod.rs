//! Native `@docker` builtin tests (soft-skip when daemon down).

mod docker_errors;
mod docker_run;

use docker_run::run_docker;
use nexus::builtins::BuiltinResult;
use nexus::env::ShellEnvironment;
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
    let matches = complete(&mut buffer, &mut cursor, &ShellEnvironment::default());
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

#[test]
fn docker_ps_all_and_logs_follow_when_daemon_up() {
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
    let stopped = format!("nexus_docker_stopped_{}", std::process::id());
    let follow = format!("nexus_docker_follow_{}", std::process::id());
    let stop_ok = Command::new("docker")
        .args([
            "run",
            "--name",
            &stopped,
            "alpine:3.20",
            "echo",
            "stopped-nexus",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    let follow_ok = Command::new("docker")
        .args([
            "run",
            "-d",
            "--name",
            &follow,
            "alpine:3.20",
            "sh",
            "-c",
            "echo follow-nexus; sleep 2",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !stop_ok || !follow_ok {
        let _ = Command::new("docker")
            .args(["rm", "-f", &stopped, &follow])
            .status();
        return;
    }

    let (ps_run, ps_run_out, _) = run_docker(&["ps"]);
    let (ps_all, ps_all_out, ps_all_err) = run_docker(&["ps", "-a"]);
    let (log_f, log_f_out, log_f_err) = run_docker(&["logs", "-f", &follow]);
    let _ = Command::new("docker")
        .args(["rm", "-f", &stopped, &follow])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    assert_eq!(ps_run, BuiltinResult::Status(0));
    assert!(
        !ps_run_out.contains(&stopped),
        "running ps should omit exited: {ps_run_out}"
    );
    assert_eq!(ps_all, BuiltinResult::Status(0), "stderr={ps_all_err}");
    assert!(
        ps_all_out.contains(&stopped),
        "ps -a missing stopped: {ps_all_out}"
    );
    assert_eq!(log_f, BuiltinResult::Status(0), "stderr={log_f_err}");
    assert!(
        log_f_out.contains("follow-nexus"),
        "log_out={log_f_out} err={log_f_err}"
    );
}

#[test]
fn docker_ps_quiet_when_daemon_up() {
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
    let name = format!("nexus_docker_quiet_{}", std::process::id());
    let status = Command::new("docker")
        .args([
            "run",
            "-d",
            "--rm",
            "--name",
            &name,
            "alpine:3.20",
            "sleep",
            "30",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    if !status.map(|s| s.success()).unwrap_or(false) {
        return;
    }
    let id = Command::new("docker")
        .args(["inspect", "-f", "{{.Id}}", &name])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().chars().take(12).collect::<String>())
        .unwrap_or_default();

    let (result, out, err) = run_docker(&["ps", "-q"]);
    let _ = Command::new("docker")
        .args(["rm", "-f", &name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    assert_eq!(result, BuiltinResult::Status(0), "stderr={err}");
    assert!(
        !out.contains("CONTAINER ID"),
        "quiet should omit header: {out}"
    );
    assert!(
        !id.is_empty() && out.lines().any(|line| line == id),
        "expected short id {id} in {out}"
    );
}
