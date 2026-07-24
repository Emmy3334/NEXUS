//! Match collection for Tab completion contexts.

use super::context::{self, Kind};
use super::paths::{collect_file_matches, collect_path_commands};
use super::{
    docker, docker_host, git, heal, helm, interp, kube, kubectl, subcmds, systemctl, vars,
};
use crate::builtins::NAMES;

pub(super) fn collect_matches(before: &str, prefix: &str, var_names: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    if vars::is_var_token(prefix) {
        vars::collect(prefix, var_names, &mut out);
        out.sort();
        out.dedup();
        return out;
    }
    let words: Vec<&str> = before.split_whitespace().collect();
    collect_kind(context::classify(before), &words, prefix, &mut out);
    out.sort();
    out.dedup();
    out
}

fn collect_kind(kind: Kind, words: &[&str], prefix: &str, out: &mut Vec<String>) {
    match kind {
        Kind::GitVerb(verb) => {
            git::collect_for_verb(verb, prefix, out);
            if out.is_empty() && !prefix.starts_with('-') && !git::is_branch_verb(verb) {
                default_matches(prefix, out);
            }
        }
        Kind::KubectlVerb(verb) => {
            kubectl::collect_for_verb(verb, words, prefix, out);
            fallback_default(prefix, out);
        }
        Kind::DockerVerb(verb) => {
            docker_host::collect_for_verb(verb, prefix, out);
            fallback_default(prefix, out);
        }
        Kind::HelmVerb(verb) => {
            helm::collect_for_verb(verb, prefix, out);
            fallback_default(prefix, out);
        }
        Kind::SystemctlVerb(verb) => {
            systemctl::collect_for_verb(verb, prefix, out);
            fallback_default(prefix, out);
        }
        Kind::Subcommand(names) => subcmds::collect(names, prefix, out),
        Kind::Interpreter { extensions } => interp::collect(prefix, extensions, out),
        Kind::Docker(kind) => docker::collect(&kind, prefix, out),
        Kind::Kube(kind) => kube::collect(&kind, prefix, out),
        Kind::Heal(kind) => heal::collect(&kind, prefix, out),
        Kind::Default => default_matches(prefix, out),
    }
}

fn fallback_default(prefix: &str, out: &mut Vec<String>) {
    if out.is_empty() && !prefix.starts_with('-') {
        default_matches(prefix, out);
    }
}

fn default_matches(prefix: &str, out: &mut Vec<String>) {
    if prefix.contains('/') || prefix.starts_with('.') {
        collect_file_matches(prefix, out);
        return;
    }
    for name in NAMES {
        if name.starts_with(prefix) {
            out.push((*name).to_owned());
        }
    }
    collect_path_commands(prefix, out);
    collect_file_matches(prefix, out);
}
