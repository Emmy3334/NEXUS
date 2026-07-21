//! How external commands attach stdout (TTY inherit vs `` `…` `` capture).

/// Whether child processes inherit the process stdout or pipe into the shell writer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StdoutMode {
    /// Keep a real TTY so tools like `ls` stay columnar.
    Inherit,
    /// Pipe into the shell `Write` (command substitution).
    Capture,
}
