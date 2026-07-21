//! Print forms of the `history` builtin.

use super::args::PrintOpts;
use crate::history::History;

use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn print_history(
    history: &History,
    opts: &PrintOpts,
    stdout: &mut impl Write,
) -> io::Result<u8> {
    let total = history.len();
    let start = match opts.last_n {
        Some(n) if n < total => total - n,
        _ => 0,
    };
    let mut items: Vec<(usize, &crate::history::HistoryEntry)> =
        history.iter().skip(start).collect();
    if opts.reverse {
        items.reverse();
    }
    for (n, entry) in items {
        write_entry(n, entry, opts, stdout)?;
    }
    Ok(0)
}

fn write_entry(
    n: usize,
    entry: &crate::history::HistoryEntry,
    opts: &PrintOpts,
    stdout: &mut impl Write,
) -> io::Result<()> {
    if opts.timestamps {
        let secs = unix_secs(entry.time);
        writeln!(stdout, "#{secs}")?;
    }
    if opts.hide_numbers {
        writeln!(stdout, "{}", entry.line)?;
    } else {
        writeln!(stdout, "{n:>5}  {}", entry.line)?;
    }
    Ok(())
}

fn unix_secs(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
