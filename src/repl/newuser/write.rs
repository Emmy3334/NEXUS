//! Persist `.nexusrc` choices from the wizard.

use super::config::{Config, Keymap, PromptStyle};

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const START: &str = "# Lines configured by nexus-newuser-install";
const END: &str = "# End of lines configured by nexus-newuser-install";

pub fn minimal(path: &Path) -> io::Result<()> {
    fs::write(
        path,
        format!(
            "# Created by nexus-newuser-install for {}\n",
            env!("CARGO_PKG_VERSION")
        ),
    )
}

pub fn configured(path: &Path, cfg: &Config) -> io::Result<()> {
    let mut body = String::new();
    body.push_str(&format!(
        "# Created by nexus-newuser-install for {}\n{START}\n",
        env!("CARGO_PKG_VERSION")
    ));
    body.push_str(&format!("set histsize = {}\n", cfg.histsize));
    match cfg.prompt_style {
        PromptStyle::Classic => {}
        PromptStyle::Powerlevel10k => {
            body.push_str("setenv NEXUS_PROMPT_STYLE powerlevel10k\n");
            body.push_str("setenv NEXUS_PROMPT_ICONS nerdfont\n");
        }
    }
    match cfg.keymap {
        Keymap::Emacs => body.push_str("bindkey -e\n"),
        Keymap::Vi => body.push_str("bindkey -v\n"),
    }
    if cfg.oh_my_nexus {
        body.push_str("setenv NEXUS_OMN \"$HOME/.oh-my-nexus\"\n");
        body.push_str("set omn_theme = powerlevel10k\n");
        body.push_str("set omn_plugins = \"git directories aliases\"\n");
        body.push_str("source $NEXUS_OMN/oh-my-nexus.nexus\n");
    }
    body.push_str(END);
    body.push('\n');
    fs::write(path, body)
}

pub(super) fn recommended(
    stdout: &mut impl Write,
    stderr: &mut impl Write,
    path: &Path,
) -> io::Result<()> {
    let Some(src) = recommended_source() else {
        writeln!(stderr, "nexus-newuser-install: no recommended file found.")?;
        return Ok(());
    };
    fs::copy(&src, path)?;
    writeln!(
        stdout,
        "Wrote recommended configuration to {}.",
        path.display()
    )?;
    Ok(())
}

/// Path to a recommended `.nexusrc`, if one exists on this system.
pub(super) fn recommended_source() -> Option<PathBuf> {
    if let Ok(explicit) = std::env::var("NEXUS_RECOMMENDED_RC") {
        let p = PathBuf::from(explicit);
        if p.is_file() {
            return Some(p);
        }
    }
    let candidates = [
        "/etc/nexus/newuser.nexusrc.recommended",
        "/usr/local/share/nexus/newuser.nexusrc.recommended",
    ];
    candidates
        .into_iter()
        .map(PathBuf::from)
        .find(|p| p.is_file())
        .or_else(embedded_recommended_path)
}

fn embedded_recommended_path() -> Option<PathBuf> {
    // Dev tree / checkout: StartupFiles next to the crate.
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("StartupFiles/nexusrc.recommended");
    path.is_file().then_some(path)
}
