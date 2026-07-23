//! `@kube` builtin and Kubernetes API helpers.

use nexus::builtins::{self, BuiltinResult};
use nexus::env::ShellEnvironment;
use nexus::kube;
use nexus::repl::complete;

fn run_kube(args: &[&str]) -> (BuiltinResult, String, String) {
    let mut env = ShellEnvironment::default();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let argv: Vec<String> = std::iter::once("@kube")
        .chain(args.iter().copied())
        .map(str::to_owned)
        .collect();
    let result = builtins::try_run(&argv, &mut env, 0, &mut stdout, &mut stderr)
        .unwrap()
        .unwrap();
    (
        result,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

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
fn kube_list_soft_fails_without_cluster() {
    if kube::cluster_reachable() {
        return;
    }
    for args in [&["pods"][..], &["nodes"][..], &["pods", "-A"][..]] {
        let (result, _, err) = run_kube(args);
        assert_eq!(result, BuiltinResult::Status(1), "args={args:?}");
        assert!(!err.is_empty(), "args={args:?}");
    }
}

#[test]
fn kube_lists_when_cluster_up() {
    if !kube::cluster_reachable() {
        return;
    }
    for args in [&["pods"][..], &["nodes"][..], &["pods", "-A"][..]] {
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
fn complete_kube_logs_soft_fails() {
    let mut buf = String::from("@kube logs ");
    let mut cursor = buf.len();
    let _ = complete(&mut buf, &mut cursor);
    if !kube::cluster_reachable() {
        assert!(kube::list_pod_names("").is_empty());
    }
}
