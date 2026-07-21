//! NEXUS integration tests — single crate root for rust-analyzer.
//!
//! Every test file under `tests/integration/` is reached from this module tree.

mod builtins;
mod exec;
mod expand;
mod glob;
mod lex;
mod parse;
mod repl;
mod shell_env;
