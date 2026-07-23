//! Heal container env pass policy (`heal_env` / `NEXUS_HEAL_ENV`).

use crate::env::ShellEnvironment;

use std::collections::BTreeSet;

/// Which exported keys may enter Docker/Kube heal containers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvPass {
    /// Forward nothing (secure default).
    None,
    /// Forward all exported keys that pass the loader-path scrub.
    All,
    /// Forward only these names (still scrubbed).
    Allow(BTreeSet<String>),
}

impl EnvPass {
    /// Resolve from `heal_env` local, else `NEXUS_HEAL_ENV`, else [`Self::None`].
    #[must_use]
    pub fn from_shell(shell_env: &ShellEnvironment) -> Self {
        if let Some(raw) = shell_env.lookup("heal_env") {
            return Self::parse(raw);
        }
        match std::env::var("NEXUS_HEAL_ENV") {
            Ok(raw) => Self::parse(&raw),
            Err(_) => Self::None,
        }
    }

    /// `none` / empty → none; `*` / `all` → all; else comma-separated allowlist.
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
            return Self::None;
        }
        if trimmed == "*" || trimmed.eq_ignore_ascii_case("all") {
            return Self::All;
        }
        let keys: BTreeSet<String> = trimmed
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect();
        if keys.is_empty() {
            Self::None
        } else {
            Self::Allow(keys)
        }
    }

    /// Status / doctor label: `none`, `all`, or `FOO,BAR`.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::None => "none".to_owned(),
            Self::All => "all".to_owned(),
            Self::Allow(keys) => keys.iter().cloned().collect::<Vec<_>>().join(","),
        }
    }

    #[must_use]
    pub fn allows(&self, key: &str) -> bool {
        match self {
            Self::None => false,
            Self::All => true,
            Self::Allow(keys) => keys.contains(key),
        }
    }
}
