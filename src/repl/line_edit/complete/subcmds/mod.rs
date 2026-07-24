//! Static first-verb lists for common external CLIs.

mod build;
mod cloud;
mod containers;
mod data;
mod k8s;
mod lang;
mod system;
mod vcs;

/// Subcommand names for `cmd`, if registered.
#[must_use]
pub(super) fn names_for(cmd: &str) -> Option<&'static [&'static str]> {
    vcs::names(cmd)
        .or_else(|| lang::names(cmd))
        .or_else(|| containers::names(cmd))
        .or_else(|| k8s::names(cmd))
        .or_else(|| cloud::names(cmd))
        .or_else(|| system::names(cmd))
        .or_else(|| build::names(cmd))
        .or_else(|| data::names(cmd))
}

/// Completing the first non-option word after a registered command.
#[must_use]
pub(super) fn first_verb(words: &[&str]) -> Option<&'static [&'static str]> {
    let cmd = words.first().copied()?;
    let names = names_for(cmd)?;
    if words.iter().skip(1).all(|w| w.starts_with('-')) {
        Some(names)
    } else {
        None
    }
}

pub(super) fn collect(names: &[&str], prefix: &str, out: &mut Vec<String>) {
    use super::matchers::matches_prefix;
    for name in names {
        if matches_prefix(name, prefix) {
            out.push((*name).to_owned());
        }
    }
}
