//! Match collection for Tab completion contexts.

mod cloud;
mod default;

use super::context::{self, Kind};
use super::vars;
use crate::env::CompRegistry;

pub(super) fn collect_matches(
    before: &str,
    prefix: &str,
    var_names: &[String],
    registry: &CompRegistry,
    path: &str,
    cmd_names: &[String],
) -> Vec<String> {
    let mut out = Vec::new();
    if vars::is_var_token(prefix) {
        vars::collect(prefix, var_names, &mut out);
        out.sort();
        out.dedup();
        return out;
    }
    let words: Vec<&str> = before.split_whitespace().collect();
    collect_kind(
        context::classify(before, registry),
        &words,
        prefix,
        registry,
        path,
        cmd_names,
        &mut out,
    );
    out.sort();
    out.dedup();
    out
}

fn collect_kind(
    kind: Kind,
    words: &[&str],
    prefix: &str,
    registry: &CompRegistry,
    path: &str,
    cmd_names: &[String],
    out: &mut Vec<String>,
) {
    match kind {
        Kind::RegisteredFirstVerb => {
            if let Some(cmd) = words.first() {
                registry.collect(cmd, prefix, out);
            }
        }
        Kind::GitVerb(verb) => cloud::git_verb(verb, prefix, path, cmd_names, out),
        Kind::Subcommand(names) => super::subcmds::collect(names, prefix, out),
        Kind::Interpreter { extensions } => super::interp::collect(prefix, extensions, out),
        Kind::Docker(kind) => super::docker::collect(&kind, prefix, out),
        Kind::Kube(kind) => super::kube::collect(&kind, prefix, out),
        Kind::Heal(kind) => super::heal::collect(&kind, prefix, out),
        Kind::Default => default::matches(prefix, path, cmd_names, words.is_empty(), out),
        other => cloud::dispatch(other, words, prefix, out),
    }
}
