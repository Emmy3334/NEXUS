//! Extra `@kube` error-path coverage.

use super::kube_run::run_kube;
use nexus::builtins::BuiltinResult;
use nexus::kube;

#[test]
fn kube_describe_unknown_resource() {
    let (result, _, err) = run_kube(&["describe", "service", "x"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("unknown"));
}

#[test]
fn kube_logs_namespace_flag_requires_value() {
    let (result, _, err) = run_kube(&["logs", "-n"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("usage:"));
}

#[test]
fn kube_pods_n_soft_fails_without_cluster() {
    if kube::cluster_reachable() {
        return;
    }
    let (result, _, err) = run_kube(&["pods", "-n", "kube-system"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(!err.is_empty());
}

#[test]
fn kube_get_pods_n_equals_form() {
    if kube::cluster_reachable() {
        let (result, out, err) = run_kube(&["get", "pods", "-n=default"]);
        assert_eq!(result, BuiltinResult::Status(0), "stderr={err}");
        assert!(out.contains("NAME") || err.contains("no resources"));
        return;
    }
    let (result, _, err) = run_kube(&["get", "pods", "-n=default"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(!err.is_empty());
}
