//! Register the Wasm cache heal backend.

use super::WasmResolver;
use crate::env::ShellEnvironment;
use crate::heal::ResolverChain;

use std::sync::Arc;

/// Attach [`WasmResolver`] as the sole heal backend (tests / explicit setup).
pub fn attach_wasm_backend(shell_env: &mut ShellEnvironment) {
    shell_env.healers = ResolverChain::from_resolvers(vec![Arc::new(WasmResolver)]);
}
