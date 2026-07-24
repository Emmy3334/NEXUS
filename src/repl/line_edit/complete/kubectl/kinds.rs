//! Resource kinds for `kubectl get` / `describe`.

const GET_RESOURCES: &[&str] = &[
    "pods",
    "nodes",
    "services",
    "deployments",
    "namespaces",
    "configmaps",
    "secrets",
    "ingresses",
    "statefulsets",
    "daemonsets",
    "jobs",
    "cronjobs",
    "replicasets",
    "endpoints",
    "pv",
    "pvc",
    "sa",
    "roles",
    "rolebindings",
    "clusterroles",
    "clusterrolebindings",
];

const DESCRIBE_RESOURCES: &[&str] = &[
    "pod",
    "node",
    "service",
    "deployment",
    "namespace",
    "configmap",
    "secret",
    "ingress",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum KindComplete {
    GetResource,
    DescribeResource,
}

/// True when completing a resource kind (no non-flag arg after get/describe yet).
pub(super) fn awaiting(words: &[&str]) -> Option<KindComplete> {
    let pos = words
        .iter()
        .position(|w| matches!(*w, "get" | "describe"))?;
    if words[pos + 1..].iter().any(|w| !w.starts_with('-')) {
        return None;
    }
    match words[pos] {
        "get" => Some(KindComplete::GetResource),
        "describe" => Some(KindComplete::DescribeResource),
        _ => None,
    }
}

pub(super) fn collect(kind: &KindComplete, prefix: &str, out: &mut Vec<String>) {
    let items = match kind {
        KindComplete::GetResource => GET_RESOURCES,
        KindComplete::DescribeResource => DESCRIBE_RESOURCES,
    };
    for item in items {
        if item.starts_with(prefix) {
            out.push((*item).to_owned());
        }
    }
}
