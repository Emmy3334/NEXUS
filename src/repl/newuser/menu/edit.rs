//! Sub-editors for the main new-user menu.

use super::super::config::{Keymap, PromptStyle};
use super::super::key;

use std::io::{self, BufRead, Write};

pub(super) fn edit_histsize(
    stdin: &mut impl BufRead,
    stdout: &mut impl Write,
    current: u32,
) -> io::Result<u32> {
    writeln!(
        stdout,
        "Enter history size (histsize), or press Enter to keep {current}:"
    )?;
    stdout.flush()?;
    let mut line = String::new();
    stdin.read_line(&mut line)?;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(current);
    }
    Ok(trimmed.parse().unwrap_or(current))
}

pub(super) fn edit_prompt(
    stdin: &mut impl BufRead,
    stdout: &mut impl Write,
) -> io::Result<PromptStyle> {
    writeln!(
        stdout,
        "(c)  Classic  $> [branch*]
(p)  Powerlevel10k-inspired segments (needs a Nerd Font)"
    )?;
    Ok(match key::read_key(stdin, stdout)? {
        'p' | 'P' => PromptStyle::Powerlevel10k,
        _ => PromptStyle::Classic,
    })
}

pub(super) fn edit_keymap(stdin: &mut impl BufRead, stdout: &mut impl Write) -> io::Result<Keymap> {
    writeln!(
        stdout,
        "(e)  Emacs bindings (default)
(v)  Vi bindings"
    )?;
    Ok(match key::read_key(stdin, stdout)? {
        'v' | 'V' => Keymap::Vi,
        _ => Keymap::Emacs,
    })
}

pub(super) fn prompt_label(style: PromptStyle) -> &'static str {
    match style {
        PromptStyle::Classic => "classic",
        PromptStyle::Powerlevel10k => "powerlevel10k",
    }
}

pub(super) fn keymap_label(map: Keymap) -> &'static str {
    match map {
        Keymap::Emacs => "emacs",
        Keymap::Vi => "vi",
    }
}
