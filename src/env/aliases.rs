//! Alias map accessors on [`ShellEnvironment`].

use super::ShellEnvironment;

impl ShellEnvironment {
    /// Borrow the body of alias `name`, if set.
    pub fn alias_get(&self, name: &str) -> Option<&str> {
        self.aliases.get(name).map(String::as_str)
    }

    /// Insert or replace alias `name`.
    pub fn alias_set(&mut self, name: impl Into<String>, body: impl Into<String>) {
        self.aliases.insert(name.into(), body.into());
    }

    /// Remove alias `name`. Returns whether it was present.
    pub fn alias_unset(&mut self, name: &str) -> bool {
        self.aliases.remove(name).is_some()
    }

    /// Iterate aliases in sorted name order as `(&str, &str)`.
    pub fn iter_aliases(&self) -> impl Iterator<Item = (&str, &str)> {
        self.aliases.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}
