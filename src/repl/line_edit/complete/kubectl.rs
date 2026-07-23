//! Host `kubectl` second-level completion (resource kinds).

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
pub(super) enum Complete {
    GetResource,
    DescribeResource,
}

/// Classify host `kubectl` (not `@kube`) when completing a resource kind.
pub(super) fn classify(words: &[&str]) -> Option<Complete> {
    if words.first().copied() != Some("kubectl") {
        return None;
    }
    let pos = words
        .iter()
        .position(|w| matches!(*w, "get" | "describe"))?;
    if words[pos + 1..].iter().any(|w| !w.starts_with('-')) {
        return None;
    }
    match words[pos] {
        "get" => Some(Complete::GetResource),
        "describe" => Some(Complete::DescribeResource),
        _ => None,
    }
}

pub(super) fn collect(kind: &Complete, prefix: &str, out: &mut Vec<String>) {
    let items = match kind {
        Complete::GetResource => GET_RESOURCES,
        Complete::DescribeResource => DESCRIBE_RESOURCES,
    };
    for item in items {
        if item.starts_with(prefix) {
            out.push((*item).to_owned());
        }
    }
}
