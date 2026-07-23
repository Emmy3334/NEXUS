//! List variable names for Tab completion and tooling.

use super::ShellEnvironment;

use std::collections::BTreeSet;

impl ShellEnvironment {
    /// Unique local ∪ exported names, sorted.
    #[must_use]
    pub fn var_names(&self) -> Vec<String> {
        let mut names = BTreeSet::new();
        names.extend(self.locals.keys().cloned());
        names.extend(self.vars.keys().cloned());
        names.into_iter().collect()
    }
}
