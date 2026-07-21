//! Owned shell environment (Minishell1: copy of the process environ).
//!
//! Builtins mutate this map; external commands inherit it. The process
//! environ is not rewritten except for the working directory (`cd`).

use std::collections::BTreeMap;

/// Live shell environment variables, keyed in sorted order for stable `env`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShellEnvironment {
    vars: BTreeMap<String, String>,
}

impl ShellEnvironment {
    /// Snapshot the current process environment.
    ///
    /// Non-UTF-8 keys/values from the OS are skipped (`std::env::vars`).
    pub fn capture() -> Self {
        Self {
            vars: std::env::vars().collect(),
        }
    }

    /// Build from an explicit map (tests / controlled setups).
    pub fn from_map(vars: BTreeMap<String, String>) -> Self {
        Self { vars }
    }

    /// Borrow the value of `name`, if set.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.vars.get(name).map(String::as_str)
    }

    /// Insert or replace `name`.
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.vars.insert(name.into(), value.into());
    }

    /// Remove `name`. Returns whether it was present.
    pub fn unset(&mut self, name: &str) -> bool {
        self.vars.remove(name).is_some()
    }

    /// Whether `name` is present (even if empty).
    pub fn contains(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }

    /// Iterate assignments in sorted key order as `(&str, &str)` pairs.
    ///
    /// Suitable for [`std::process::Command::envs`] without extra `OsString`s.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.vars.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}
