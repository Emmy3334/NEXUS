//! Positional / script arguments (`$0`, `$1`, `$#`, `$*`).

use super::ShellEnvironment;

impl ShellEnvironment {
    /// Replace the shell's positional vector (`argv[0]` is `$0`).
    pub fn set_argv(&mut self, argv: impl Into<Vec<String>>) {
        self.argv = argv.into();
    }

    /// Borrow the positional vector (`$0` at index 0).
    #[must_use]
    pub fn argv(&self) -> &[String] {
        &self.argv
    }

    /// `$n` — `None` if out of range (expands to empty).
    #[must_use]
    pub fn positional(&self, n: usize) -> Option<&str> {
        self.argv.get(n).map(String::as_str)
    }

    /// `$#` — count of `$1`… (excludes `$0`).
    #[must_use]
    pub fn argc(&self) -> usize {
        self.argv.len().saturating_sub(1)
    }

    /// `$*` — `$1`… joined with spaces.
    #[must_use]
    pub fn star(&self) -> String {
        self.argv
            .iter()
            .skip(1)
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ")
    }
}
