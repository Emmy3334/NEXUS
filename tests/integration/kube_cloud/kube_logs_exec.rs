//! Usage / soft-fail paths for `@kube logs -f` and `@kube exec`.

use super::kube_run::run_kube;
use nexus::builtins::BuiltinResult;
use nexus::kube;

#[test]
fn kube_logs_follow_requires_pod() {
    let (result, _, err) = run_kube(&["logs", "-f"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("usage:"));
}

#[test]
fn kube_exec_requires_pod_and_cmd() {
    let (result, _, err) = run_kube(&["exec"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("usage:"));

    let (result, _, err) = run_kube(&["exec", "some-pod"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("usage:"));
}

#[test]
fn kube_exec_soft_fails_without_cluster() {
    if kube::cluster_reachable() {
        return;
    }
    let (result, _, err) = run_kube(&["exec", "nope", "--", "true"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(!err.is_empty());
}

#[test]
fn kube_logs_follow_soft_fails_without_cluster() {
    if kube::cluster_reachable() {
        return;
    }
    let (result, _, err) = run_kube(&["logs", "-f", "nope"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(!err.is_empty());
}
