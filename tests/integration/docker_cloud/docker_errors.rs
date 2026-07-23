//! Extra `@docker` error-path coverage.

use super::docker_run::run_docker;
use nexus::builtins::BuiltinResult;

#[test]
fn docker_ps_rejects_extra_args() {
    let (result, _, err) = run_docker(&["ps", "extra"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("usage:"));
}

#[test]
fn docker_logs_rejects_extra_args() {
    let (result, _, err) = run_docker(&["logs", "a", "b"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("usage:"));
}
