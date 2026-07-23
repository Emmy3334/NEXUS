//! Ephemeral Kubernetes Pod heal backend (kube-rs).

mod missing;
mod register;
mod run;

pub use register::attach_kube_backend;

use super::banner;
use super::config;
use super::env_pairs;
use super::CommandResolver;
use crate::env::ShellEnvironment;
use crate::kube;
use crate::tokio_rt::block_on;
use missing::should_decline;

use std::io::{self, Write};

/// Default image for missing-command healing (matches Docker heal).
pub const DEFAULT_IMAGE: &str = "alpine:3.20";

/// Runs a missing command inside an ephemeral Pod.
pub struct KubeResolver {
    image: String,
}

impl KubeResolver {
    /// Probe the cluster; `None` when kubeconfig / API is unreachable.
    #[must_use]
    pub fn probe(image: impl Into<String>) -> Option<Self> {
        if !kube::cluster_reachable() {
            return None;
        }
        Some(Self {
            image: image.into(),
        })
    }
}

impl CommandResolver for KubeResolver {
    fn try_heal(
        &self,
        argv: &[String],
        shell_env: &mut ShellEnvironment,
        stdin: Option<&[u8]>,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
    ) -> io::Result<Option<u8>> {
        if argv.is_empty() || stdin.map(|b| !b.is_empty()).unwrap_or(false) {
            // Non-empty stdin: decline so Docker can attach bytes (no Pod attach yet).
            return Ok(None);
        }
        let argv0 = argv.first().map(String::as_str).unwrap_or("");
        if !super::image_map::container_heal_allowed(argv0, shell_env) {
            return Ok(None);
        }
        let env = env_pairs::kube_env(shell_env);
        let image = super::image_map::image_for(argv0, &self.image);
        tracing::debug!(argv0, backend = "kube", %image, "heal try_heal");
        map_run(
            block_on(run::execute(image, argv, env, stdout, stderr)),
            argv0,
            shell_env,
            stderr,
        )
    }
}

fn map_run(
    ran: io::Result<io::Result<(u8, String)>>,
    argv0: &str,
    shell_env: &ShellEnvironment,
    stderr: &mut dyn Write,
) -> io::Result<Option<u8>> {
    match ran {
        Ok(Ok((127, _))) => Ok(None),
        Ok(Ok((code, name))) => {
            tracing::info!(argv0, backend = "kube", status = code, pod = %name, "heal success");
            let detail = format!("pod {name}");
            banner::success(stderr, config::quiet_from(shell_env), "kube", Some(&detail))?;
            Ok(Some(code))
        }
        Ok(Err(err)) | Err(err) if should_decline(&err) => Ok(None),
        Ok(Err(err)) | Err(err) => {
            tracing::warn!(argv0, backend = "kube", error = %err, "heal failed");
            writeln!(stderr, "nexus: kube heal failed: {err}")?;
            Ok(Some(1))
        }
    }
}
