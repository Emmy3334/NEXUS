//! Soft load/save of session history for interactive TTY shells.

use crate::env::ShellEnvironment;
use crate::history;

use std::io::{self, Write};
use std::path::PathBuf;

/// Append `~/.nexus_history` (or `histfile`) into memory. Missing file is quiet.
pub fn load_session_history<E: Write>(
    shell_env: &mut ShellEnvironment,
    stderr: &mut E,
) -> io::Result<()> {
    let path = histfile(shell_env);
    match shell_env.history.load_append(&path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => {
            writeln!(stderr, "history: {}: {err}.", path.display())?;
            Ok(())
        }
    }
}

/// Write the session list to the histfile. Empty + missing path skips create.
pub fn save_session_history<E: Write>(
    shell_env: &ShellEnvironment,
    stderr: &mut E,
) -> io::Result<()> {
    let path = histfile(shell_env);
    if shell_env.history.is_empty() && !path.exists() {
        return Ok(());
    }
    if let Err(err) = shell_env.history.save_to(&path) {
        writeln!(stderr, "history: {}: {err}.", path.display())?;
    }
    Ok(())
}

fn histfile(shell_env: &ShellEnvironment) -> PathBuf {
    history::histfile_path(
        shell_env.get_local("histfile"),
        shell_env
            .lookup("home")
            .or_else(|| shell_env.lookup("HOME")),
    )
}
