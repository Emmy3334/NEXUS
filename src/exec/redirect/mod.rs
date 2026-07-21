//! File and heredoc redirections for simple commands and pipeline stages.
//!
//! File / heredoc redirects override a pipe on the same fd. Heredoc bodies are
//! collected from the shell input stream after parse (see
//! [`collect_heredoc_bodies`]).

mod child_io;
mod files;
mod heredoc;
mod stage;

pub use heredoc::collect_heredoc_bodies;

pub(super) use files::{apply_stdout_for_stage, open_redirect_files, RedirectFiles, StdinSource};
pub(super) use heredoc::HeredocState;
pub(super) use stage::execute_simple;
