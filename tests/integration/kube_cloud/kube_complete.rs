//! Tab completion for `@kube`.

use nexus::kube;
use nexus::repl::complete;

#[test]
fn complete_kube_logs_soft_fails() {
    let mut buf = String::from("@kube logs ");
    let mut cursor = buf.len();
    let _ = complete(&mut buf, &mut cursor);
    if !kube::cluster_reachable() {
        assert!(kube::list_pod_names("", None).is_empty());
        assert!(kube::list_namespace_names("").is_empty());
    }
}

#[test]
fn complete_kube_subcommand() {
    let mut buf = String::from("@kube ");
    let mut cursor = buf.len();
    let matches = complete(&mut buf, &mut cursor);
    // Ambiguous → list; or single shared prefix applied into buffer.
    let joined = format!("{buf}{}", matches.join(" "));
    assert!(
        joined.contains("pods") || joined.contains("logs") || joined.contains("describe"),
        "buf={buf} matches={matches:?}"
    );
}

#[test]
fn complete_kube_get_resource() {
    let mut buf = String::from("@kube get ");
    let mut cursor = buf.len();
    let matches = complete(&mut buf, &mut cursor);
    let joined = format!("{buf}{}", matches.join(" "));
    assert!(
        joined.contains("pods") || joined.contains("nodes"),
        "buf={buf} matches={matches:?}"
    );
}
