//! Owned shell environment (Minishell1: copy of the process environ).
//!
//! Builtins mutate the exported map; external commands inherit only that map.
//! Non-exported shell locals are consulted first during `$` expansion.
//! Special locals (`cwd`, `home`, `user`, `term`) are seeded at capture time.
//! The process environ is not rewritten except for the working directory (`cd`).

use std::collections::BTreeMap;
use std::path::Path;

/// Live shell environment: exported vars plus optional non-exported locals.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShellEnvironment {
    /// Exported / environ copy — inherited by children.
    vars: BTreeMap<String, String>,
    /// Shell-local variables (not passed to children).
    locals: BTreeMap<String, String>,
}

impl ShellEnvironment {
    /// Snapshot the current process environment and seed special locals.
    ///
    /// Non-UTF-8 keys/values from the OS are skipped (`std::env::vars`).
    pub fn capture() -> Self {
        let mut env = Self {
            vars: std::env::vars().collect(),
            locals: BTreeMap::new(),
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
        }
    }

    /// Seed / refresh `cwd`, `home`, `user`, and `term` shell locals.
    ///
    /// Values come from the live process cwd and exported `HOME` / `USER` / `TERM`
    /// when present. Existing locals with the same names are overwritten.
    pub fn seed_specials(&mut self) {
        if let Ok(cwd) = std::env::current_dir() {
            self.set_local("cwd", path_string(&cwd));
        }
        if let Some(home) = self.get("HOME") {
            self.set_local("home", home.to_owned());
        }
        if let Some(user) = self.get("USER") {
            self.set_local("user", user.to_owned());
        }
        if let Some(term) = self.get("TERM") {
            self.set_local("term", term.to_owned());
        }
    }

    /// Update the `cwd` local (and typically paired with exported `PWD`).
    pub fn set_cwd(&mut self, path: impl Into<String>) {
        self.set_local("cwd", path);
    }

    /// Lookup for `$` expansion: **local first**, then exported env.
    pub fn lookup(&self, name: &str) -> Option<&str> {
        self.locals
            .get(name)
            .or_else(|| self.vars.get(name))
            .map(String::as_str)
    }

    /// Borrow the exported value of `name`, if set (ignores locals).
    pub fn get(&self, name: &str) -> Option<&str> {
        self.vars.get(name).map(String::as_str)
    }

    /// Borrow a local value of `name`, if set (ignores exported).
    pub fn get_local(&self, name: &str) -> Option<&str> {
        self.locals.get(name).map(String::as_str)
    }

    /// Insert or replace exported `name`.
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.vars.insert(name.into(), value.into());
    }

    /// Insert or replace a non-exported local.
    pub fn set_local(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.locals.insert(name.into(), value.into());
    }

    /// Remove exported `name`. Returns whether it was present.
    pub fn unset(&mut self, name: &str) -> bool {
        self.vars.remove(name).is_some()
    }

    /// Remove a local `name`. Returns whether it was present.
    pub fn unset_local(&mut self, name: &str) -> bool {
        self.locals.remove(name).is_some()
    }

    /// Whether exported `name` is present (even if empty).
    pub fn contains(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }

    /// Iterate **exported** assignments in sorted key order as `(&str, &str)`.
    ///
    /// Suitable for [`std::process::Command::envs`] without extra `OsString`s.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.vars.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Iterate **local** assignments in sorted key order as `(&str, &str)`.
    pub fn iter_locals(&self) -> impl Iterator<Item = (&str, &str)> {
        self.locals.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
