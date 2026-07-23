//! Ephemeral Docker container heal backend (bollard).

mod client;
mod cmd_logs;
mod list;
mod missing;
mod ps;
mod register;
mod run;
mod runtime;

pub use cmd_logs::write_container_logs;
pub use list::running_names;
pub use ps::list_ps_lines;
pub use register::attach_docker_backend;

use super::banner;
use super::config;
use super::env_pairs;
use super::CommandResolver;
use crate::env::ShellEnvironment;
use client::Docker;
use missing::should_decline;
use runtime::block_on;

use std::io::{self, Write};

/// Default image for missing-command healing.
pub const DEFAULT_IMAGE: &str = "alpine:3.20";

/// True when the local Docker daemon accepts a ping.
#[must_use]
pub fn daemon_reachable() -> bool {
    let Ok(docker) = client::connect() else {
        return false;
    };
    matches!(block_on(client::ping(&docker)), Ok(Ok(())))
}

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
        shell_env: &mut ShellEnvironment,
        stdin: Option<&[u8]>,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
    ) -> io::Result<Option<u8>> {
        if argv.is_empty() {
            return Ok(None);
        }
        let argv0 = argv.first().map(String::as_str).unwrap_or("");
        let env = env_pairs::docker_env(shell_env);
        tracing::debug!(argv0, backend = "docker", "heal try_heal");
        let ran = block_on(run::execute(
            &self.docker,
            &self.image,
            argv,
            &env,
            stdin,
            stdout,
            stderr,
        ));
        map_run(ran, argv0, shell_env, stderr)
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
        Ok(Ok((code, id))) => {
            tracing::info!(argv0, backend = "docker", status = code, %id, "heal success");
            banner::success(stderr, config::quiet_from(shell_env), "docker", Some(&id))?;
            Ok(Some(code))
        }
        Ok(Err(err)) | Err(err) if should_decline(&err) => Ok(None),
        Ok(Err(err)) | Err(err) => {
            tracing::warn!(argv0, backend = "docker", error = %err, "heal failed");
            writeln!(stderr, "nexus: docker heal failed: {err}")?;
            Ok(Some(1))
        }
    }
}
