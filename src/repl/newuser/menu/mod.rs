//! Main configuration menu (simplified vs zsh-newuser-install).

mod edit;

use super::config::Config;
use super::key;
use edit::{edit_histsize, edit_keymap, edit_prompt, keymap_label, prompt_label};

use std::io::{self, BufRead, Write};

/// Run the menu; `None` means quit without writing.
pub(super) fn run(
    stdin: &mut impl BufRead,
    stdout: &mut impl Write,
    zdmsg: &str,
) -> io::Result<Option<Config>> {
    let mut cfg = Config::default();
    loop {
        clear(stdout)?;
        print_menu(stdout, &cfg, zdmsg)?;
        match key::read_key(stdin, stdout)? {
            '1' => cfg.histsize = edit_histsize(stdin, stdout, cfg.histsize)?,
            '2' => cfg.prompt_style = edit_prompt(stdin, stdout)?,
            '3' => cfg.keymap = edit_keymap(stdin, stdout)?,
            '4' => cfg.oh_my_nexus = !cfg.oh_my_nexus,
            '0' => return Ok(Some(cfg)),
            'q' | 'Q' => return Ok(None),
            _ => {
                writeln!(stdout, "Unknown key.")?;
                let _ = key::read_key(stdin, stdout)?;
            }
        }
    }
}

fn print_menu(stdout: &mut impl Write, cfg: &Config, zdmsg: &str) -> io::Result<()> {
    let omn = if cfg.oh_my_nexus { "on" } else { "off" };
    let omn_verb = if cfg.oh_my_nexus { "Keep" } else { "Enable" };
    writeln!(
        stdout,
        "Please pick one of the following options:

(1)  Configure history size (currently {}).

(2)  Configure the primary prompt style (currently {}).

(3)  Configure line-editing keymap (currently {}).

(4)  {omn_verb} Oh My Nexus in .nexusrc (currently {omn}).

(0)  Exit, saving the new settings to {zdmsg}/.nexusrc.

(q)  Quit and do nothing else.  The function will be run again next time.",
        cfg.histsize,
        prompt_label(cfg.prompt_style),
        keymap_label(cfg.keymap),
    )
}

fn clear(stdout: &mut impl Write) -> io::Result<()> {
    write!(stdout, "\x1b[H\x1b[2J")?;
    stdout.flush()
}
