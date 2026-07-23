//! NEXUS integration tests — single crate root for rust-analyzer.
//!
//! Every test file under `tests/integration/` is reached from this module tree.

mod advanced;
mod builtins;
mod complete_context;
mod exec;
mod expand;
mod foreach;
mod git_prompt;
mod glob;
mod history_persist;
mod if_block;
mod jobs_signals;
mod kube_cloud;
mod lex;
mod line_edit;
mod line_edit_paste;
mod nexusrc;
mod observability;
mod parse;
mod repl;
mod sandbox_wasm;
mod scripting;
mod shell_env;
mod specials;
mod while_loop;
