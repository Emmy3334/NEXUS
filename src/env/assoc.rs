//! Associative shell arrays (`typeset -A`).

use super::ShellEnvironment;

use std::collections::BTreeMap;

impl ShellEnvironment {
    /// Whether `name` is an associative array (possibly empty).
    pub fn is_assoc(&self, name: &str) -> bool {
        self.assoc.contains_key(name)
    }

    /// Value for `key` in associative array `name`.
    pub fn assoc_get(&self, name: &str, key: &str) -> Option<&str> {
        self.assoc.get(name)?.get(key).map(String::as_str)
    }

    /// Replace associative array `name` with `map` (clears conflicting scalars/arrays).
    pub fn assoc_set(&mut self, name: impl Into<String>, map: BTreeMap<String, String>) {
        let name = name.into();
        self.locals.remove(&name);
        self.vars.remove(&name);
        self.arrays.remove(&name);
        self.assoc.insert(name, map);
    }

    /// Sorted keys of associative array `name` (empty when unset).
    pub fn assoc_keys(&self, name: &str) -> Vec<String> {
        self.assoc
            .get(name)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Values of associative array `name` in key order (empty when unset).
    pub fn assoc_values(&self, name: &str) -> Vec<String> {
        self.assoc
            .get(name)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Number of entries in associative array `name` (`0` when unset).
    pub fn assoc_len(&self, name: &str) -> usize {
        self.assoc.get(name).map_or(0, BTreeMap::len)
    }

    /// Declare an associative array at function scope (restore on leave).
    pub(crate) fn declare_assoc(&mut self, name: &str, map: BTreeMap<String, String>) {
        let prior = self.assoc.get(name).cloned();
        if let Some(frame) = self.local_frames.last_mut() {
            frame.saved_assoc.entry(name.to_owned()).or_insert(prior);
        }
        self.assoc_set(name, map);
    }
}
