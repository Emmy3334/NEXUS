//! First-run setup wizard (zsh `zsh-newuser-install` analogue).

mod config;
mod detect;
mod intro;
mod key;
mod menu;
mod write;

pub use config::{Config, Keymap, PromptStyle};
pub use detect::should_offer;
pub use write::{configured as write_configured_rc, minimal as write_minimal_rc};

use crate::env::ShellEnvironment;
use crate::repl::rc::path;

use std::io::{self, BufRead, Write};

/// Run the wizard when appropriate; may create `$dotdir/.nexusrc`.
pub(super) fn maybe_run<I: BufRead, O: Write, E: Write>(
    stdin: &mut I,
    stdout: &mut O,
    stderr: &mut E,
    shell_env: &ShellEnvironment,
) -> io::Result<()> {
    if !should_offer(shell_env) {
        return Ok(());
    }
    let Some(dotdir) = path::resolve_dotdir(shell_env) else {
        return Ok(());
    };
    let zdmsg = display_dotdir(&dotdir);
    match intro::run(stdin, stdout, &dotdir, &zdmsg)? {
        intro::Choice::Quit => Ok(()),
        intro::Choice::Minimal => write::minimal(&dotdir.join(".nexusrc")),
        intro::Choice::Recommended => write::recommended(stdout, stderr, &dotdir.join(".nexusrc")),
        intro::Choice::MainMenu => {
            let Some(cfg) = menu::run(stdin, stdout, &zdmsg)? else {
                return Ok(());
            };
            write::configured(&dotdir.join(".nexusrc"), &cfg)
        }
    }
}

fn display_dotdir(dotdir: &std::path::Path) -> String {
    if let Ok(home) = std::env::var("HOME") {
        if let Ok(rest) = dotdir.strip_prefix(&home) {
            if rest.as_os_str().is_empty() {
                return "~".into();
            }
            return format!("~/{}", rest.display());
        }
    }
    dotdir.display().to_string()
}
