//! Ephemeral Docker container heal backend (bollard).

mod client;
mod list;
mod missing;
mod register;
mod run;
mod runtime;

pub use list::running_names;
pub use register::attach_docker_backend;

use super::CommandResolver;
use crate::env::ShellEnvironment;
use client::Docker;
use missing::is_missing_in_image;
use runtime::block_on;

use std::io::{self, Write};

/// Default image for missing-command healing.
pub const DEFAULT_IMAGE: &str = "alpine:3.20";

/// Runs a missing command inside an ephemeral Docker container.
pub struct DockerResolver {
    docker: Docker,
    image: String,
}

impl DockerResolver {
    /// Connect and ping the local Docker daemon; `None` if unavailable.
    #[must_use]
    pub fn probe(image: impl Into<String>) -> Option<Self> {
        let docker = client::connect().ok()?;
        match block_on(client::ping(&docker)) {
            Ok(Ok(())) => {}
            _ => return None,
        }
        Some(Self {
            docker,
            image: image.into(),
        })
    }
}

impl CommandResolver for DockerResolver {
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
        match block_on(run::execute(
            &self.docker,
            &self.image,
            argv,
            stdout,
            stderr,
        )) {
            Ok(Ok(127)) => Ok(None),
            Ok(Ok(code)) => Ok(Some(code)),
            Ok(Err(err)) | Err(err) if is_missing_in_image(&err) => Ok(None),
            Ok(Err(err)) | Err(err) => {
                writeln!(stderr, "nexus: docker heal failed: {err}")?;
                Ok(Some(1))
            }
        }
    }
}
