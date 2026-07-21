//! Read-only lookups into the exported / local variable maps.

use super::ShellEnvironment;

impl ShellEnvironment {
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
