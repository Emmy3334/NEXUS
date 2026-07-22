//! Self-healing command resolution after local spawn `NotFound`.
//!
//! When no resolvers are registered, behavior matches classic shells:
//! print `{cmd}: Command not found.` and return `127`.

mod after_spawn;
mod chain;
mod docker;
mod report;

pub use after_spawn::{after_spawn_failure, after_spawn_failure_os};
pub use chain::ResolverChain;
pub use docker::{attach_docker_backend, running_names, DockerResolver, DEFAULT_IMAGE};
pub(crate) use report::report_spawn_failure;

use crate::env::ShellEnvironment;

use std::io::{self, Write};

/// Attach built-in heal backends that are available on this host.
pub fn attach_default_backends(shell_env: &mut ShellEnvironment) {
    attach_docker_backend(shell_env);
}

/// Backend that may recover a command missing on the local host.
///
/// Return `Ok(Some(status))` when the command was handled, `Ok(None)` to try
/// the next resolver (or fall through to the classic not-found message).
pub trait CommandResolver: Send + Sync {
    fn try_heal(
        &self,
        argv: &[String],
        shell_env: &mut ShellEnvironment,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
    ) -> io::Result<Option<u8>>;
}
