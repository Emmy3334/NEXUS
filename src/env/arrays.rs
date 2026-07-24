//! Shell array variables (`typeset -a`).

use super::ShellEnvironment;

impl ShellEnvironment {
    /// Whether `name` is an array (possibly empty).
    pub fn is_array(&self, name: &str) -> bool {
        self.arrays.contains_key(name)
    }

    /// Borrow array elements when `name` is an array.
    pub fn array_get(&self, name: &str) -> Option<&[String]> {
        self.arrays.get(name).map(Vec::as_slice)
    }

    /// Replace array `name` with `elements` (clears conflicting scalars).
    pub fn array_set(&mut self, name: impl Into<String>, elements: Vec<String>) {
        let name = name.into();
        self.locals.remove(&name);
        self.vars.remove(&name);
        self.arrays.insert(name, elements);
    }

    /// Element count for an array; `0` when unset.
    pub fn array_len(&self, name: &str) -> usize {
        self.arrays.get(name).map_or(0, Vec::len)
    }

    /// Join array elements with a single space.
    pub fn array_join(&self, name: &str) -> String {
        self.arrays
            .get(name)
            .map(|v| v.join(" "))
            .unwrap_or_default()
    }

    /// Declare an array at function scope (restore on leave).
    pub(crate) fn declare_array(&mut self, name: &str, elements: Vec<String>) {
        let prior = self.arrays.get(name).cloned();
        if let Some(frame) = self.local_frames.last_mut() {
            frame.saved_arrays.entry(name.to_owned()).or_insert(prior);
        }
        self.array_set(name, elements);
    }
}
