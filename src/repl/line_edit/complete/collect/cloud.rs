//! Cloud / host-tool completion branches.

use super::super::context::Kind;
use super::super::{aws, docker_host, gcloud, git, helm, kubectl, systemctl};
use super::default;

pub(super) fn git_verb(
    verb: &str,
    prefix: &str,
    path: &str,
    cmd_names: &[String],
    out: &mut Vec<String>,
) {
    git::collect_for_verb(verb, prefix, out);
    if out.is_empty() && !prefix.starts_with('-') && !git::is_branch_verb(verb) {
        // Not first-token; prefer files over re-listing PATH.
        default::matches(prefix, path, cmd_names, false, out);
    }
}

pub(super) fn dispatch(kind: Kind, words: &[&str], prefix: &str, out: &mut Vec<String>) {
    match kind {
        Kind::KubectlVerb(verb) => {
            kubectl::collect_for_verb(verb, words, prefix, out);
            default::fallback(prefix, out);
        }
        Kind::DockerVerb(verb) => {
            docker_host::collect_for_verb(verb, prefix, out);
            default::fallback(prefix, out);
        }
        Kind::HelmVerb(verb) => {
            helm::collect_for_verb(verb, prefix, out);
            default::fallback(prefix, out);
        }
        Kind::SystemctlVerb(verb) => {
            systemctl::collect_for_verb(verb, prefix, out);
            default::fallback(prefix, out);
        }
        Kind::AwsVerb(verb) => {
            aws::collect_for_verb(verb, words, prefix, out);
            default::fallback(prefix, out);
        }
        Kind::GcloudVerb(verb) => {
            gcloud::collect_for_verb(verb, words, prefix, out);
            default::fallback(prefix, out);
        }
        _ => {}
    }
}
