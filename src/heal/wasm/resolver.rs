//! Heal missing commands via the local Wasm module cache.

use super::super::banner;
use super::super::config;
use super::super::CommandResolver;
use crate::env::ShellEnvironment;
use crate::sandbox;

use std::io::{self, Write};

/// Runs `argv[0]` from `~/.nexus/wasm/<name>.wasm` when present.
pub struct WasmResolver;

impl CommandResolver for WasmResolver {
    fn try_heal(
        &self,
        argv: &[String],
        shell_env: &mut ShellEnvironment,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
    ) -> io::Result<Option<u8>> {
        let Some(name) = argv.first().map(String::as_str) else {
            return Ok(None);
        };
        let Some(path) = sandbox::resolve_named(name) else {
            return Ok(None);
        };
        let args = &argv[1..];
        tracing::debug!(argv0 = name, backend = "wasm", "heal try_heal");
        let code = sandbox::run_module(&path, args, stdout, stderr)?;
        tracing::info!(
            argv0 = name,
            backend = "wasm",
            status = code,
            "heal success"
        );
        banner::success(stderr, config::quiet_from(shell_env), "wasm", Some(name))?;
        Ok(Some(code))
    }
}
