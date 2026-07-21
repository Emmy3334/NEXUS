//! Owned shell environment (Minishell1: copy of the process environ).
//!
//! Builtins mutate this map; external commands inherit it. The process
//! environ is not rewritten except for the working directory (`cd`).

use std::collections::BTreeMap;
use std::ffi::OsString;

/// Live shell environment variables, keyed in sorted order for stable `env`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShellEnvironment {
    vars: BTreeMap<String, String>,
}

impl ShellEnvironment {
    /// Snapshot the current process environment.
    pub fn capture() -> Self {
        Self {
            vars: std::env::vars().collect(),
        }
    }

    /// Build from an explicit map (tests / controlled setups).
    pub fn from_map(vars: BTreeMap<String, String>) -> Self {
        Self { vars }
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.vars.get(name).map(String::as_str)
    }

    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.vars.insert(name.into(), value.into());
    }

    pub fn unset(&mut self, name: &str) -> bool {
        self.vars.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.vars.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Pairs suitable for [`std::process::Command::envs`].
    pub fn command_envs(&self) -> impl Iterator<Item = (OsString, OsString)> + '_ {
        self.vars
            .iter()
            .map(|(k, v)| (OsString::from(k), OsString::from(v)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_unset() {
        let mut env = ShellEnvironment::default();
        env.set("FOO", "bar");
        assert_eq!(env.get("FOO"), Some("bar"));
        assert!(env.unset("FOO"));
        assert_eq!(env.get("FOO"), None);
    }
}
