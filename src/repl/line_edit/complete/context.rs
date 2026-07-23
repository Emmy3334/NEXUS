//! Classify the completion context from words before the current token.

use super::{docker, heal, kube};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Kind {
    Default,
    GitBranch,
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
    if git_branch_context(&words) {
        return Kind::GitBranch;
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

fn git_branch_context(words: &[&str]) -> bool {
    if words.first().copied() != Some("git") {
        return false;
    }
    words
        .iter()
        .skip(1)
        .any(|w| matches!(*w, "checkout" | "switch" | "branch"))
}
