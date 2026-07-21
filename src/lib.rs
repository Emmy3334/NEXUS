//! NEXUS library — shell language processor (lex → parse → exec).
//!
//! The `nexus` binary is a thin driver over this crate. Domain logic lives
//! here so it stays reusable and testable without going through `main`.

pub mod alias;
pub mod builtins;
pub mod env;
pub mod exec;
pub mod expand;
pub mod glob;
pub mod lex;
pub mod parse;
pub mod repl;
