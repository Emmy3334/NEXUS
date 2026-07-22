//! Process-wide shared Wasmtime engine.

use std::sync::OnceLock;
use wasmtime::Engine;

/// Shared [`Engine`] for module compile / instantiate (avoids per-call setup).
#[must_use]
pub(super) fn shared() -> &'static Engine {
    static ENGINE: OnceLock<Engine> = OnceLock::new();
    ENGINE.get_or_init(Engine::default)
}
