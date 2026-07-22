//! Register Docker heal backend when the daemon is reachable.

use super::{DockerResolver, DEFAULT_IMAGE};
use crate::env::ShellEnvironment;
use crate::heal::ResolverChain;

use std::sync::Arc;

/// Attach [`DockerResolver`] when Docker responds to ping; otherwise no-op.
pub fn attach_docker_backend(shell_env: &mut ShellEnvironment) {
    let Some(resolver) = DockerResolver::probe(DEFAULT_IMAGE) else {
        return;
    };
    shell_env.healers = ResolverChain::from_resolvers(vec![Arc::new(resolver)]);
}
