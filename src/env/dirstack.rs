//! Directory stack for `pushd` / `popd` / `dirs`.

use std::path::PathBuf;

/// Stack of directories (index 0 is the top / current after pushd).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DirStack {
    entries: Vec<PathBuf>,
}

impl DirStack {
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn push(&mut self, path: PathBuf) {
        self.entries.insert(0, path);
    }

    pub fn pop(&mut self) -> Option<PathBuf> {
        if self.entries.is_empty() {
            None
        } else {
            Some(self.entries.remove(0))
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &PathBuf> {
        self.entries.iter()
    }

    /// Ensure stack has at least cwd when first used.
    pub fn ensure_seeded(&mut self, cwd: PathBuf) {
        if self.entries.is_empty() {
            self.entries.push(cwd);
        }
    }
}
