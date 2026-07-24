//! Classify the completion context from words before the current token.

use super::{aws, docker, docker_host, gcloud, git, heal, helm, kube, kubectl, subcmds, systemctl};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Kind {
    Default,
    GitVerb(&'static str),
    KubectlVerb(&'static str),
    DockerVerb(&'static str),
    HelmVerb(&'static str),
    SystemctlVerb(&'static str),
    AwsVerb(&'static str),
    GcloudVerb(&'static str),
    Subcommand(&'static [&'static str]),
    Interpreter { extensions: &'static [&'static str] },
    Docker(docker::Complete),
    Kube(kube::Complete),
    Heal(heal::Complete),
}

/// Inspect whitespace-separated words before the token being completed.
///
/// Subcommands may be followed by flags (e.g. `git checkout -b `, `@docker logs -f `).
#[must_use]
pub(super) fn classify(before: &str) -> Kind {
    let words: Vec<&str> = before.split_whitespace().collect();
    if let Some(verb) = git::verb_in(&words) {
        return Kind::GitVerb(verb);
    }
    if let Some(verb) = kubectl::verb_in(&words) {
        return Kind::KubectlVerb(verb);
    }
    if let Some(verb) = docker_host::verb_in(&words) {
        return Kind::DockerVerb(verb);
    }
    if let Some(verb) = helm::verb_in(&words) {
        return Kind::HelmVerb(verb);
    }
    if let Some(verb) = systemctl::verb_in(&words) {
        return Kind::SystemctlVerb(verb);
    }
    if let Some(verb) = aws::verb_in(&words) {
        return Kind::AwsVerb(verb);
    }
    if let Some(verb) = gcloud::verb_in(&words) {
        return Kind::GcloudVerb(verb);
    }
    if let Some(names) = subcmds::first_verb(&words) {
        return Kind::Subcommand(names);
    }
    if let Some(docker) = docker::classify(&words) {
        return Kind::Docker(docker);
    }
    if let Some(kube) = kube::classify(&words) {
        return Kind::Kube(kube);
    }
    if let Some(heal) = heal::classify(&words) {
        return Kind::Heal(heal);
    }
    match words.first().copied() {
        Some("python" | "python3") => Kind::Interpreter {
            extensions: &[".py"],
        },
        Some("ruby") => Kind::Interpreter {
            extensions: &[".rb"],
        },
        _ => Kind::Default,
    }
}
