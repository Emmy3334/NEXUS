//! Kubernetes-aware Tab completion for `@kube`.

use crate::kube;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Complete {
    Subcommand,
    GetResource,
    DescribeResource,
    Namespace,
    Pod { namespace: Option<String> },
}

const SUBCOMMANDS: &[&str] = &["nodes", "pods", "logs", "exec", "get", "describe", "help"];
const GET_RESOURCES: &[&str] = &["pods", "nodes"];
const DESCRIBE_RESOURCES: &[&str] = &["pod"];

/// Classify `@kube` completion when the line starts with `@kube`.
pub(super) fn classify(words: &[&str]) -> Option<Complete> {
    if words.first().copied() != Some("@kube") {
        return None;
    }
    if after_ns_flag(words) {
        return Some(Complete::Namespace);
    }
    match words.get(1).copied() {
        None => Some(Complete::Subcommand),
        Some("get") if words.len() == 2 => Some(Complete::GetResource),
        Some("describe") if words.len() == 2 => Some(Complete::DescribeResource),
        Some("logs" | "exec") => Some(Complete::Pod {
            namespace: ns_from(words),
        }),
        Some("describe") if words.get(2) == Some(&"pod") || words.get(2) == Some(&"pods") => {
            Some(Complete::Pod {
                namespace: ns_from(words),
            })
        }
        _ => None,
    }
}

pub(super) fn collect(kind: &Complete, prefix: &str, out: &mut Vec<String>) {
    match kind {
        Complete::Subcommand => push_static(SUBCOMMANDS, prefix, out),
        Complete::GetResource => push_static(GET_RESOURCES, prefix, out),
        Complete::DescribeResource => push_static(DESCRIBE_RESOURCES, prefix, out),
        Complete::Namespace => out.extend(kube::list_namespace_names(prefix)),
        Complete::Pod { namespace } => {
            out.extend(kube::list_pod_names(prefix, namespace.as_deref()));
        }
    }
}

fn push_static(items: &[&str], prefix: &str, out: &mut Vec<String>) {
    for item in items {
        if item.starts_with(prefix) {
            out.push((*item).to_owned());
        }
    }
}

fn after_ns_flag(words: &[&str]) -> bool {
    matches!(words.last().copied(), Some("-n" | "--namespace"))
}

fn ns_from(words: &[&str]) -> Option<String> {
    let mut i = 0;
    while i < words.len() {
        match words[i] {
            "-n" | "--namespace" => {
                return words.get(i + 1).map(|s| (*s).to_owned());
            }
            flag if flag.starts_with("-n=") || flag.starts_with("--namespace=") => {
                return flag.split_once('=').map(|(_, v)| v.to_owned());
            }
            _ => i += 1,
        }
    }
    None
}
