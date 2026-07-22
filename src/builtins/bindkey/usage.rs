//! `bindkey -u` usage text.

use std::io::{self, Write};

pub(super) fn print(stdout: &mut impl Write) -> io::Result<u8> {
    writeln!(
        stdout,
        "Usage: bindkey [-l|-d|-e|-v|-u]\n       bindkey [-a] [-b] [-k] [-r] [--] key\n       bindkey [-a] [-b] [-k] [-c|-s] [--] key command"
    )?;
    Ok(0)
}
