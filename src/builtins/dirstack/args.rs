//! Shared flag parsing for `dirs` / `pushd` / `popd`.

/// How the directory stack should be printed.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct PrintFlags {
    /// Expand `~` to the full home path (`-l`).
    pub(super) long: bool,
    /// One entry per line with indices (`-v`); wins over wrap.
    pub(super) verbose: bool,
    /// Wrap the horizontal listing before `COLUMNS` (`-n`).
    pub(super) wrap: bool,
}

/// Parse a stack index of the form `+n`.
pub(super) fn parse_plus_index(raw: &str) -> Option<usize> {
    let rest = raw.strip_prefix('+')?;
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    rest.parse().ok()
}

/// Consume leading `-…` print flags; returns remaining argv and flags.
pub(super) fn take_print_flags(args: &[String]) -> Result<(PrintFlags, &[String]), String> {
    let mut flags = PrintFlags::default();
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg == "--" {
            return Ok((flags, &args[i + 1..]));
        }
        let Some(rest) = arg.strip_prefix('-') else {
            break;
        };
        if rest.is_empty() || rest.bytes().all(|b| b.is_ascii_digit()) {
            break;
        }
        apply_print_chars(rest, &mut flags)?;
        i += 1;
    }
    Ok((flags, &args[i..]))
}

fn apply_print_chars(chars: &str, flags: &mut PrintFlags) -> Result<(), String> {
    for ch in chars.chars() {
        match ch {
            'l' => flags.long = true,
            'v' => flags.verbose = true,
            'n' => flags.wrap = true,
            'p' => {}
            _ => return Err(format!("Unknown option: -{ch}.")),
        }
    }
    Ok(())
}
