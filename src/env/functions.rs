//! Shell function table accessors on [`ShellEnvironment`].

use super::ShellEnvironment;

impl ShellEnvironment {
    /// Borrow the body of function `name`, if defined.
    pub fn function_get(&self, name: &str) -> Option<&str> {
        self.functions.get(name).map(String::as_str)
    }

    /// Insert or replace function `name`.
    pub fn function_set(&mut self, name: impl Into<String>, body: impl Into<String>) {
        self.functions.insert(name.into(), body.into());
    }

    /// Remove function `name`. Returns whether it was present.
    pub fn function_unset(&mut self, name: &str) -> bool {
        self.functions.remove(name).is_some()
    }

    /// Iterate functions in sorted name order as `(&str, &str)`.
    pub fn iter_functions(&self) -> impl Iterator<Item = (&str, &str)> {
        self.functions.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}
