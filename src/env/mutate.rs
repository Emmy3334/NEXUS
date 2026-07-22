//! Insertion / removal on the exported and local variable maps.

use super::ShellEnvironment;

impl ShellEnvironment {
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
}
