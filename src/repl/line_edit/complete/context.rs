//! Classify the completion context from words before the current token.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Default,
    GitBranch,
    Interpreter { extensions: &'static [&'static str] },
    DockerContainer,
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
    if docker_logs_context(&words) {
        return Kind::DockerContainer;
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

fn docker_logs_context(words: &[&str]) -> bool {
    words.first().copied() == Some("@docker") && words.iter().skip(1).any(|w| *w == "logs")
}
