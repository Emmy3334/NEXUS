//! `history` builtin — print, save/load/merge, and clear.

mod args;
mod file_ops;
mod print;

use crate::env::ShellEnvironment;
use args::{parse_argv, HistoryAction};
use file_ops::{clear_history, load_history, merge_history, save_history};
use print::print_history;

use std::io::{self, Write};

pub(super) fn history_cmd(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match parse_argv(argv) {
        Ok(HistoryAction::Print(opts)) => print_history(&shell_env.history, &opts, stdout),
        Ok(HistoryAction::Clear) => {
            clear_history(shell_env);
            Ok(0)
        }
        Ok(HistoryAction::Save(path)) => save_history(shell_env, path.as_deref(), stderr),
        Ok(HistoryAction::Load(path)) => load_history(shell_env, path.as_deref(), stderr),
        Ok(HistoryAction::Merge(path)) => merge_history(shell_env, path.as_deref(), stderr),
        Err(msg) => {
            writeln!(stderr, "{msg}")?;
            Ok(1)
        }
    }
}
