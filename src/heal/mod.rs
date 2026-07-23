//! Self-healing command resolution after local spawn `NotFound`.
//!
//! When no resolvers are registered, behavior matches classic shells:
//! print `{cmd}: Command not found.` and return `127`.

mod after_spawn;
mod banner;
mod chain;
mod config;
mod docker;
mod kube;
mod report;
mod wasm;

pub use after_spawn::{after_spawn_failure, after_spawn_failure_os};
pub use chain::ResolverChain;
pub use config::{parse_order, quiet_from, resolve as resolve_settings, Backend, Settings};
pub use docker::{attach_docker_backend, running_names, DockerResolver, DEFAULT_IMAGE};
pub use kube::{attach_kube_backend, KubeResolver, DEFAULT_IMAGE as KUBE_DEFAULT_IMAGE};
pub(crate) use report::report_spawn_failure;
pub use wasm::{attach_wasm_backend, WasmResolver};

use crate::env::ShellEnvironment;
use config::Backend as HealBackend;

use std::io::{self, Write};
use std::sync::Arc;

/// Attach built-in heal backends that are available on this host.
///
/// Default order: Wasm → Kube → Docker (override via `heal_order` / `NEXUS_HEAL_ORDER`).
pub fn attach_default_backends(shell_env: &mut ShellEnvironment) {
    let settings = config::resolve(shell_env);
    let mut resolvers: Vec<Arc<dyn CommandResolver>> = Vec::new();
    for backend in &settings.order {
        match backend {
            HealBackend::Wasm => resolvers.push(Arc::new(WasmResolver)),
            HealBackend::Docker => {
                if let Some(docker) = DockerResolver::probe(settings.image.clone()) {
                    resolvers.push(Arc::new(docker));
                }
            }
            HealBackend::Kube => {
                if let Some(kube) = KubeResolver::probe(settings.image.clone()) {
                    resolvers.push(Arc::new(kube));
                }
            }
        }
    }
    shell_env.healers = ResolverChain::from_resolvers(resolvers);
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
