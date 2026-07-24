//! Cloud / host-tool completion branches.

use super::super::context::Kind;
use super::super::{aws, docker_host, gcloud, git, helm, kubectl, systemctl};
use super::default;

pub(super) fn git_verb(verb: &str, prefix: &str, path: &str, out: &mut Vec<String>) {
    git::collect_for_verb(verb, prefix, out);
    if out.is_empty() && !prefix.starts_with('-') && !git::is_branch_verb(verb) {
        default::matches(prefix, path, out);
    }
}

pub(super) fn dispatch(
    kind: Kind,
    words: &[&str],
    prefix: &str,
    path: &str,
    out: &mut Vec<String>,
) {
    match kind {
        Kind::KubectlVerb(verb) => {
            kubectl::collect_for_verb(verb, words, prefix, out);
            default::fallback(prefix, path, out);
        }
        Kind::DockerVerb(verb) => {
            docker_host::collect_for_verb(verb, prefix, out);
            default::fallback(prefix, path, out);
        }
        Kind::HelmVerb(verb) => {
            helm::collect_for_verb(verb, prefix, out);
            default::fallback(prefix, path, out);
        }
        Kind::SystemctlVerb(verb) => {
            systemctl::collect_for_verb(verb, prefix, out);
            default::fallback(prefix, path, out);
        }
        Kind::AwsVerb(verb) => {
            aws::collect_for_verb(verb, words, prefix, out);
            default::fallback(prefix, path, out);
        }
        Kind::GcloudVerb(verb) => {
            gcloud::collect_for_verb(verb, words, prefix, out);
            default::fallback(prefix, path, out);
        }
        _ => {}
    }
}
