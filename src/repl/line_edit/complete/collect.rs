//! Match collection for Tab completion contexts.

use super::context::{self, Kind};
use super::paths::{collect_file_matches, collect_path_commands};
use super::{docker, git, heal, interp, kube, kubectl, subcmds, vars};
use crate::builtins::NAMES;

pub(super) fn collect_matches(before: &str, prefix: &str, var_names: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    if vars::is_var_token(prefix) {
        vars::collect(prefix, var_names, &mut out);
        out.sort();
        out.dedup();
        return out;
    }
    match context::classify(before) {
        Kind::GitVerb(verb) => {
            git::collect_for_verb(verb, prefix, &mut out);
            if out.is_empty() && !prefix.starts_with('-') && !git::is_branch_verb(verb) {
                default_matches(prefix, &mut out);
            }
        }
        Kind::Kubectl(kind) => kubectl::collect(&kind, prefix, &mut out),
        Kind::Subcommand(names) => subcmds::collect(names, prefix, &mut out),
        Kind::Interpreter { extensions } => interp::collect(prefix, extensions, &mut out),
        Kind::Docker(kind) => docker::collect(&kind, prefix, &mut out),
        Kind::Kube(kind) => kube::collect(&kind, prefix, &mut out),
        Kind::Heal(kind) => heal::collect(&kind, prefix, &mut out),
        Kind::Default => default_matches(prefix, &mut out),
    }
    out.sort();
    out.dedup();
    out
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
