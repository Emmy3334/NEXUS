//! Seeding / refreshing the special shell locals (`cwd`, `home`, `user`, `term`).

use super::ShellEnvironment;

use std::path::Path;

impl ShellEnvironment {
    /// Seed / refresh `cwd`, `home`, `user`, and `term` shell locals.
    ///
    /// Values come from the live process cwd and exported `HOME` / `USER` / `TERM`
    /// when present. Existing locals with the same names are overwritten.
    pub fn seed_specials(&mut self) {
        if let Ok(cwd) = std::env::current_dir() {
            self.set_local("cwd", path_string(&cwd));
        }
        if let Some(home) = self.get("HOME") {
            self.set_local("home", home.to_owned());
        }
        if let Some(user) = self.get("USER") {
            self.set_local("user", user.to_owned());
        }
        if let Some(term) = self.get("TERM") {
            self.set_local("term", term.to_owned());
        }
    }

    /// Update the `cwd` local (and typically paired with exported `PWD`).
    pub fn set_cwd(&mut self, path: impl Into<String>) {
        self.set_local("cwd", path);
    }
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
