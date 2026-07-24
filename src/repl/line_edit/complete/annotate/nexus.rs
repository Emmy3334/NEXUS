//! Descriptions for NEXUS builtins (@docker, @kube, heal).

use super::super::context::Kind;
use super::super::docker;
use super::super::heal;
use super::super::kube;
use super::super::match_item::Match;

pub(super) fn apply(kind: &Kind, m: &mut Match) {
    let desc = match kind {
        Kind::Heal(heal::Complete::Subcommand) => heal_cmd(&m.value),
        Kind::Docker(d) => docker_desc(d, &m.value),
        Kind::Kube(k) => kube_desc(k, &m.value),
        _ => None,
    };
    if desc.is_some() {
        m.description = desc.map(str::to_owned);
    }
}

fn heal_cmd(value: &str) -> Option<&'static str> {
    match value {
        "help" => Some("Show heal usage and tips"),
        "status" => Some("Docker/Kube connectivity summary"),
        _ => None,
    }
}

fn docker_desc(kind: &docker::Complete, value: &str) -> Option<&'static str> {
    match kind {
        docker::Complete::Subcommand => match value {
            "ps" => Some("List containers"),
            "logs" => Some("Stream container logs"),
            "help" => Some("Show @docker usage"),
            _ => None,
        },
        docker::Complete::Logs => flag_desc(DOCKER_LOG_FLAGS, value),
        docker::Complete::Ps => flag_desc(DOCKER_PS_FLAGS, value),
        docker::Complete::Help => None,
    }
}

fn kube_desc(kind: &kube::Complete, value: &str) -> Option<&'static str> {
    match kind {
        kube::Complete::Subcommand => match value {
            "pods" => Some("List pod names"),
            "logs" => Some("Pod logs"),
            "exec" => Some("Run command in pod"),
            "get" => Some("Get resource by kind"),
            "describe" => Some("Describe resource"),
            "nodes" => Some("Cluster nodes"),
            "help" => Some("Show @kube usage"),
            _ => None,
        },
        kube::Complete::GetResource => match value {
            "pods" => Some("Pod resources"),
            "nodes" => Some("Node resources"),
            _ => None,
        },
        kube::Complete::DescribeResource => match value {
            "pod" => Some("Describe a pod"),
            _ => None,
        },
        kube::Complete::Pod { .. }
        | kube::Complete::Namespace
        | kube::Complete::Nodes
        | kube::Complete::Pods => flag_desc(KUBE_FLAGS, value),
    }
}

fn flag_desc(table: &'static [(&'static str, &'static str)], value: &str) -> Option<&'static str> {
    for (k, d) in table {
        if *k == value {
            return Some(*d);
        }
    }
    None
}

const DOCKER_PS_FLAGS: &[(&str, &str)] = &[
    ("-a", "Include stopped containers"),
    ("--all", "Include stopped containers"),
    ("-q", "Print IDs only"),
    ("--quiet", "Print IDs only"),
];

const DOCKER_LOG_FLAGS: &[(&str, &str)] = &[
    ("-f", "Follow log output"),
    ("--follow", "Follow log output"),
    ("--tail", "Number of lines from end"),
];

const KUBE_FLAGS: &[(&str, &str)] = &[
    ("-n", "Kubernetes namespace"),
    ("--namespace", "Kubernetes namespace"),
    ("-A", "All namespaces"),
    ("--all-namespaces", "All namespaces"),
    ("-f", "Follow log output"),
    ("--follow", "Follow log output"),
];
