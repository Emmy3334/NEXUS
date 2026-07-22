//! Wasm module cache heal backend.

mod register;
mod resolver;

pub use register::attach_wasm_backend;
pub use resolver::WasmResolver;
