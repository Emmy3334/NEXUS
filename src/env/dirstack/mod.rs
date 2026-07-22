//! Directory stack for `pushd` / `popd` / `dirs`.

mod ops;

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

    pub fn iter(&self) -> impl Iterator<Item = &PathBuf> {
        self.entries.iter()
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&PathBuf> {
        self.entries.get(index)
    }

    /// Ensure stack has at least cwd when first used.
    pub fn ensure_seeded(&mut self, cwd: PathBuf) {
        if self.entries.is_empty() {
            self.entries.push(cwd);
        }
    }
}
