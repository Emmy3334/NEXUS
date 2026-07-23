//! Which namespaces to query for pods / logs / describe.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PodScope {
    /// Default namespace from kubeconfig.
    Default,
    /// All namespaces (`-A`).
    All,
    /// Explicit namespace (`-n`).
    Namespace(String),
}

impl PodScope {
    #[must_use]
    pub fn with_ns_column(&self) -> bool {
        matches!(self, Self::All)
    }
}
