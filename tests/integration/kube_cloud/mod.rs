//! `@kube` builtin and Kubernetes API helpers.

mod kube_complete;
mod kube_errors;
mod kube_list;
mod kube_run;

use kube_run::run_kube;
use nexus::builtins::BuiltinResult;
use nexus::kube;

#[test]
fn kube_help_without_cluster() {
    let (result, _, err) = run_kube(&[]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("usage:"));
}

#[test]
fn kube_unknown_subcommand() {
    let (result, _, err) = run_kube(&["widgets"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("unknown"));
}

#[test]
fn kube_logs_requires_pod_name() {
    let (result, _, err) = run_kube(&["logs"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("usage:"));
}

#[test]
fn kube_describe_requires_pod_name() {
    let (result, _, err) = run_kube(&["describe", "pod"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("usage:"));
}

#[test]
fn kube_namespace_flag_requires_value() {
    let (result, _, err) = run_kube(&["pods", "-n"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("usage:"));
}

#[test]
fn kube_get_unknown_resource() {
    let (result, _, err) = run_kube(&["get", "services"]);
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(err.contains("unknown"));
}

#[test]
fn kube_list_soft_fails_without_cluster() {
    if kube::cluster_reachable() {
        return;
    }
    for args in [
        &["pods"][..],
        &["nodes"][..],
        &["pods", "-A"][..],
        &["get", "pods"][..],
        &["describe", "pod", "nope"][..],
    ] {
        let (result, _, err) = run_kube(args);
        assert_eq!(result, BuiltinResult::Status(1), "args={args:?}");
        assert!(!err.is_empty(), "args={args:?}");
    }
}
