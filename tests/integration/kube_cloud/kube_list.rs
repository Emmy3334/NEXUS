//! Cluster-up listing / describe smoke tests.

use super::kube_run::run_kube;
use nexus::builtins::BuiltinResult;
use nexus::kube;

#[test]
fn kube_lists_when_cluster_up() {
    if !kube::cluster_reachable() {
        return;
    }
    for args in [
        &["pods"][..],
        &["nodes"][..],
        &["pods", "-A"][..],
        &["get", "nodes"][..],
    ] {
        let (result, out, err) = run_kube(args);
        assert_eq!(
            result,
            BuiltinResult::Status(0),
            "args={args:?} stderr={err}"
        );
        assert!(
            out.contains("NAME") || err.contains("no resources"),
            "args={args:?} out={out} err={err}"
        );
    }
}

#[test]
fn kube_describe_soft_or_real() {
    if !kube::cluster_reachable() {
        let (result, _, err) = run_kube(&["describe", "pod", "missing-pod-xyz"]);
        assert_eq!(result, BuiltinResult::Status(1));
        assert!(!err.is_empty());
        return;
    }
    let (list, out, _) = run_kube(&["pods"]);
    if list != BuiltinResult::Status(0) || !out.contains('\n') {
        return;
    }
    let name = out
        .lines()
        .nth(1)
        .and_then(|line| line.split_whitespace().next());
    let Some(name) = name else {
        return;
    };
    let (result, desc, err) = run_kube(&["describe", "pod", name]);
    assert_eq!(result, BuiltinResult::Status(0), "stderr={err}");
    assert!(desc.contains("Name:"), "desc={desc}");
}
