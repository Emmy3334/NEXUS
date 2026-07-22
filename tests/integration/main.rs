//! NEXUS integration tests — single crate root for rust-analyzer.
//!
//! Every test file under `tests/integration/` is reached from this module tree.

mod advanced;
mod builtins;
mod exec;
mod expand;
mod foreach;
mod glob;
mod if_block;
mod jobs_signals;
mod lex;
mod line_edit;
mod parse;
mod repl;
mod scripting;
mod shell_env;
mod specials;
mod while_loop;
