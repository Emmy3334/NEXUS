//! Ordered list of [`super::CommandResolver`] backends.

use super::CommandResolver;
use crate::env::ShellEnvironment;

use std::io::{self, Write};
use std::sync::Arc;

/// Session heal backends (cheap to clone via `Arc`).
#[derive(Clone, Default)]
pub struct ResolverChain {
    resolvers: Arc<[Arc<dyn CommandResolver>]>,
}

impl std::fmt::Debug for ResolverChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolverChain")
            .field("backends", &self.resolvers.len())
            .finish()
    }
}

impl ResolverChain {
    /// Empty chain — spawn failures stay classic not-found.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Build from an ordered list of backends (first match wins).
    #[must_use]
    pub fn from_resolvers(resolvers: Vec<Arc<dyn CommandResolver>>) -> Self {
        Self {
            resolvers: resolvers.into(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.resolvers.is_empty()
    }

    /// Number of registered backends (tests / diagnostics).
    #[must_use]
    pub fn len(&self) -> usize {
        self.resolvers.len()
    }

    /// Run resolvers until one returns `Some(status)`.
    pub fn try_heal(
        &self,
        argv: &[String],
        shell_env: &mut ShellEnvironment,
        stdin: Option<&[u8]>,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
    ) -> io::Result<Option<u8>> {
        for resolver in self.resolvers.iter() {
            if let Some(code) = resolver.try_heal(argv, shell_env, stdin, stdout, stderr)? {
                return Ok(Some(code));
            }
        }
        Ok(None)
    }
}
