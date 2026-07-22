//! Heal missing commands via the local Wasm module cache.

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
        _shell_env: &mut ShellEnvironment,
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
        let code = sandbox::run_module(&path, args, stdout, stderr)?;
        Ok(Some(code))
    }
}
