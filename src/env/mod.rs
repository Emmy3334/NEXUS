//! Owned shell environment (Minishell1: copy of the process environ).
//!
//! Builtins mutate the exported map; external commands inherit only that map.
//! Non-exported shell locals are consulted first during `$` expansion.
//! Aliases and command history are shell-only (never exported to children).
//! Special locals (`cwd`, `home`, `user`, `term`) are seeded at capture time.
//! The process environ is not rewritten except for the working directory (`cd`).
//! Background jobs live here but are cleared when the env is cloned (subshells).

mod access;
mod aliases;
mod argv;
mod dirstack;
mod mutate;
mod specials;

pub use dirstack::DirStack;

use crate::heal::ResolverChain;
use crate::history::History;
use crate::jobs::JobTable;
use crate::keybind::KeyBindings;

use std::collections::BTreeMap;

/// Live shell environment: exported vars, locals, aliases, history, and jobs.
#[derive(Debug, Default)]
pub struct ShellEnvironment {
    /// Exported / environ copy — inherited by children.
    pub(super) vars: BTreeMap<String, String>,
    /// Shell-local variables (not passed to children).
    pub(super) locals: BTreeMap<String, String>,
    /// Command aliases (not passed to children).
    pub(super) aliases: BTreeMap<String, String>,
    /// Positional parameters for scripting (`$0`, `$1`, …).
    pub(super) argv: Vec<String>,
    /// Session command history for `!` events and `history`.
    pub history: History,
    /// Interactive editor bindings for `bindkey` / line edition.
    pub key_bindings: KeyBindings,
    /// Directory stack for `pushd` / `popd` / `dirs`.
    pub dir_stack: DirStack,
    /// Self-heal backends for missing external commands (empty = classic 127).
    pub healers: ResolverChain,
    /// Background jobs for `&` / `jobs` / `fg` / `bg`.
    pub jobs: JobTable,
}

impl Clone for ShellEnvironment {
    fn clone(&self) -> Self {
        Self {
            vars: self.vars.clone(),
            locals: self.locals.clone(),
            aliases: self.aliases.clone(),
            argv: self.argv.clone(),
            history: self.history.clone(),
            key_bindings: self.key_bindings.clone(),
            dir_stack: self.dir_stack.clone(),
            healers: self.healers.clone(),
            // Subshells must not inherit live child processes.
            jobs: JobTable::default(),
        }
    }
}

impl PartialEq for ShellEnvironment {
    fn eq(&self, other: &Self) -> bool {
        self.vars == other.vars
            && self.locals == other.locals
            && self.aliases == other.aliases
            && self.argv == other.argv
            && self.history == other.history
            && self.key_bindings == other.key_bindings
            && self.dir_stack == other.dir_stack
        // `jobs` / `healers` excluded: not value identity for env snapshots.
    }
}

impl Eq for ShellEnvironment {}

impl ShellEnvironment {
    /// Snapshot the current process environment and seed special locals.
    ///
    /// Non-UTF-8 keys/values from the OS are skipped (`std::env::vars`).
    pub fn capture() -> Self {
        let mut env = Self {
            vars: std::env::vars().collect(),
            locals: BTreeMap::new(),
            aliases: BTreeMap::new(),
            argv: Vec::new(),
            history: History::default(),
            key_bindings: KeyBindings::new(),
            dir_stack: DirStack::default(),
            healers: ResolverChain::empty(),
            jobs: JobTable::default(),
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
            argv: Vec::new(),
            history: History::default(),
            key_bindings: KeyBindings::new(),
            dir_stack: DirStack::default(),
            healers: ResolverChain::empty(),
            jobs: JobTable::default(),
        }
    }
}
