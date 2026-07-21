//! Owned shell environment (Minishell1: copy of the process environ).
//!
//! Builtins mutate the exported map; external commands inherit only that map.
//! Non-exported shell locals are consulted first during `$` expansion.
//! Aliases and command history are shell-only (never exported to children).
//! Special locals (`cwd`, `home`, `user`, `term`) are seeded at capture time.
//! The process environ is not rewritten except for the working directory (`cd`).

mod access;
mod aliases;
mod mutate;
mod specials;

use crate::history::History;

use std::collections::BTreeMap;

/// Live shell environment: exported vars, locals, aliases, and history.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShellEnvironment {
    /// Exported / environ copy — inherited by children.
    pub(super) vars: BTreeMap<String, String>,
    /// Shell-local variables (not passed to children).
    pub(super) locals: BTreeMap<String, String>,
    /// Command aliases (not passed to children).
    pub(super) aliases: BTreeMap<String, String>,
    /// Session command history for `!` events and `history`.
    pub history: History,
}

impl ShellEnvironment {
    /// Snapshot the current process environment and seed special locals.
    ///
    /// Non-UTF-8 keys/values from the OS are skipped (`std::env::vars`).
    pub fn capture() -> Self {
        let mut env = Self {
            vars: std::env::vars().collect(),
            locals: BTreeMap::new(),
            aliases: BTreeMap::new(),
            history: History::default(),
        };
        env.seed_specials();
        env
    }

    /// Build from an explicit exported map (tests / controlled setups).
    ///
    /// Does **not** seed specials — tests that need them call [`Self::seed_specials`].
    pub fn from_map(vars: BTreeMap<String, String>) -> Self {
        Self {
            vars,
            locals: BTreeMap::new(),
            aliases: BTreeMap::new(),
            history: History::default(),
        }
    }
}
