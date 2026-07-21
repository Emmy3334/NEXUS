//! NEXUS library — shell language processor (lex → parse → exec).
//!
//! The `nexus` binary is a thin driver over this crate. Domain logic lives
//! here so it stays reusable and testable without going through `main`.

pub mod lex;
pub mod repl;
