//! Register Kubernetes heal backend when the cluster is reachable.

use super::{KubeResolver, DEFAULT_IMAGE};
use crate::env::ShellEnvironment;
use crate::heal::ResolverChain;

use std::sync::Arc;

/// Attach [`KubeResolver`] when the API responds; otherwise no-op.
pub fn attach_kube_backend(shell_env: &mut ShellEnvironment) {
    let Some(resolver) = KubeResolver::probe(DEFAULT_IMAGE) else {
        return;
    };
    shell_env.healers = ResolverChain::from_resolvers(vec![Arc::new(resolver)]);
}
