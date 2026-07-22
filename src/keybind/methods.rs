//! Bind / lookup / unbind / list on [`KeyBindings`].

use super::{Binding, KeyBindings};

impl KeyBindings {
    /// Bind on the primary map, or alternate when `alternate` is set.
    pub fn bind(&mut self, keys: impl Into<Vec<u8>>, binding: Binding, alternate: bool) {
        let map = if alternate {
            &mut self.alternate
        } else {
            &mut self.primary
        };
        map.insert(keys.into(), binding);
    }

    #[must_use]
    pub fn lookup(&self, keys: &[u8]) -> Option<&Binding> {
        self.lookup_in(keys, self.alternate_active)
    }

    /// Look up in a specific map (for `bindkey` display / `-a`).
    #[must_use]
    pub fn lookup_in(&self, keys: &[u8], alternate: bool) -> Option<&Binding> {
        let map = if alternate {
            &self.alternate
        } else {
            &self.primary
        };
        map.get(keys)
    }

    pub fn unbind(&mut self, keys: &[u8], alternate: bool) -> bool {
        let map = if alternate {
            &mut self.alternate
        } else {
            &mut self.primary
        };
        map.remove(keys).is_some()
    }

    /// Sorted entries of the chosen map.
    #[must_use]
    pub fn sorted_entries(&self, alternate: bool) -> Vec<(Vec<u8>, Binding)> {
        let map = if alternate {
            &self.alternate
        } else {
            &self.primary
        };
        let mut entries: Vec<_> = map.iter().map(|(k, b)| (k.clone(), b.clone())).collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries
    }
}
