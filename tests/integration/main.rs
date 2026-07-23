//! NEXUS integration tests — single crate root for rust-analyzer.
//!
//! Every test file under `tests/integration/` is reached from this module tree.

mod advanced;
mod brace_glob;
mod builtins;
mod case_block;
mod complete_context;
mod complete_list;
mod cwd_lock;
mod exec;
mod expand;
mod expand_param;
mod foreach;
mod functions;
mod git_prompt;
mod glob;
mod history_persist;
mod if_block;
mod jobs_signals;
mod kube_cloud;
mod lex;
mod line_edit;
mod line_edit_isearch;
mod line_edit_paste;
mod line_edit_word;
mod nexusrc;
mod observability;
mod parse;
mod repl;
mod sandbox_wasm;
mod scripting;
mod shell_env;
mod specials;
mod while_loop;
