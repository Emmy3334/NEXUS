//! Ephemeral Kubernetes Pod heal backend (kube-rs).

mod missing;
mod register;
mod run;

pub use register::attach_kube_backend;

use super::CommandResolver;
use crate::env::ShellEnvironment;
use crate::kube;
use crate::tokio_rt::block_on;
use missing::is_missing_in_image;

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
        _shell_env: &mut ShellEnvironment,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
    ) -> io::Result<Option<u8>> {
        if argv.is_empty() {
            return Ok(None);
        }
        match block_on(run::execute(&self.image, argv, stdout, stderr)) {
            Ok(Ok(127)) => Ok(None),
            Ok(Ok(code)) => Ok(Some(code)),
            Ok(Err(err)) | Err(err) if is_missing_in_image(&err) => Ok(None),
            Ok(Err(err)) | Err(err) => {
                writeln!(stderr, "nexus: kube heal failed: {err}")?;
                Ok(Some(1))
            }
        }
    }
}
