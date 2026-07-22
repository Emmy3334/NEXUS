//! tcsh-style special aliases and `ignoreeof`.

mod ignoreeof;
mod run;

pub use ignoreeof::allow_exit_on_eof;
pub use run::{run_cwdcmd, run_precmd};
