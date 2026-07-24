//! Static descriptions for host cloud CLIs.

use super::super::context::Kind;
use super::super::match_item::Match;

pub(super) fn apply(kind: &Kind, m: &mut Match) {
    let desc = match kind {
        Kind::AwsVerb(svc) => aws_desc(svc, &m.value),
        Kind::GcloudVerb(grp) => gcloud_desc(grp, &m.value),
        Kind::HelmVerb(v) | Kind::SystemctlVerb(v) | Kind::KubectlVerb(v) => {
            host_verb_desc(kind, v, &m.value)
        }
        _ => None,
    };
    if desc.is_some() {
        m.description = desc.map(str::to_owned);
    }
}

fn aws_desc(service: &str, value: &str) -> Option<&'static str> {
    flag_desc(AWS_FLAGS, value).or(match service {
        "s3" => s3_action(value),
        _ => None,
    })
}

fn gcloud_desc(group: &str, value: &str) -> Option<&'static str> {
    flag_desc(GCLOUD_FLAGS, value).or(match group {
        "compute" => compute_cmd(value),
        _ => None,
    })
}

fn host_verb_desc(kind: &Kind, verb: &str, value: &str) -> Option<&'static str> {
    let flags = match kind {
        Kind::HelmVerb(_) => HELM_FLAGS,
        Kind::SystemctlVerb(_) => SYSTEMCTL_FLAGS,
        Kind::KubectlVerb(_) => KUBECTL_FLAGS,
        _ => return None,
    };
    flag_desc(flags, value).or(match (kind, verb, value) {
        (Kind::HelmVerb(_), "install" | "upgrade", "--namespace") => Some("Release namespace"),
        (Kind::HelmVerb(_), "install", "--create-namespace") => Some("Create namespace if missing"),
        (Kind::SystemctlVerb(_), "status", "--user") => Some("User session units"),
        (Kind::KubectlVerb(_), "get", "pods") | (Kind::KubectlVerb(_), "describe", "pods") => {
            Some("Pod resources")
        }
        (Kind::KubectlVerb(_), "get", "deployments") => Some("Deployment resources"),
        (Kind::KubectlVerb(_), "logs", "--follow" | "-f") => Some("Stream log output"),
        _ => None,
    })
}

fn flag_desc(table: &'static [(&'static str, &'static str)], value: &str) -> Option<&'static str> {
    for (k, d) in table {
        if *k == value {
            return Some(*d);
        }
    }
    None
}

fn s3_action(value: &str) -> Option<&'static str> {
    match value {
        "ls" => Some("List buckets or objects"),
        "cp" => Some("Copy local file or S3 object"),
        "sync" => Some("Sync directories or prefixes"),
        "mb" => Some("Make bucket"),
        "rb" => Some("Remove bucket"),
        "rm" => Some("Delete object"),
        "presign" => Some("Generate presigned URL"),
        _ => None,
    }
}

fn compute_cmd(value: &str) -> Option<&'static str> {
    match value {
        "instances" => Some("VM instances"),
        "ssh" => Some("SSH into an instance"),
        "disks" => Some("Persistent disks"),
        "zones" => Some("Availability zones"),
        "firewall-rules" => Some("VPC firewall rules"),
        "networks" => Some("VPC networks"),
        "images" => Some("Machine images"),
        "addresses" => Some("Static IP addresses"),
        _ => None,
    }
}

const AWS_FLAGS: &[(&str, &str)] = &[
    ("--recursive", "Apply to all objects under prefix"),
    ("--region", "AWS region"),
    ("--profile", "Named credential profile"),
    ("--output", "Output format (json, text, table)"),
    ("--dryrun", "Show actions without executing"),
];

const GCLOUD_FLAGS: &[(&str, &str)] = &[
    ("--project", "GCP project ID"),
    ("--zone", "Compute zone"),
    ("--region", "Compute region"),
    ("--quiet", "Disable interactive prompts"),
    ("--format", "Output format"),
];

const HELM_FLAGS: &[(&str, &str)] = &[
    ("--namespace", "Release namespace"),
    ("--create-namespace", "Create namespace if missing"),
    ("--values", "Values file"),
    ("--set", "Set values on command line"),
];

const SYSTEMCTL_FLAGS: &[(&str, &str)] = &[
    ("--user", "User session units"),
    ("--no-block", "Do not wait for operation finish"),
    ("--no-ask-password", "Do not prompt for password"),
];

const KUBECTL_FLAGS: &[(&str, &str)] = &[
    ("--namespace", "Kubernetes namespace"),
    ("-n", "Kubernetes namespace"),
    ("--all-namespaces", "All namespaces"),
    ("-A", "All namespaces"),
    ("--follow", "Stream log output"),
    ("-f", "Stream log output"),
];
