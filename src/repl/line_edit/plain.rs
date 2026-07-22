//! Plain `BufRead` acquisition (pipes, scripts, and non-TTY interactive).

use std::collections::VecDeque;
use std::io::{self, BufRead};

/// Read one physical line. `Ok(false)` is EOF; `Ok(true)` is a line (possibly empty).
pub(super) fn read_into(
    stdin: &mut impl BufRead,
    buffer: &mut String,
    queue: &mut VecDeque<u8>,
) -> io::Result<bool> {
    // Leftover paste bytes from the raw TTY editor (same process stdin).
    #[cfg(unix)]
    if let Some(line) = super::tty::take_complete_line(queue) {
        buffer.push_str(&line);
        return Ok(true);
    }
    #[cfg(not(unix))]
    let _ = queue;
    let bytes = stdin.read_line(buffer)?;
    if bytes == 0 {
        buffer.clear();
        return Ok(false);
    }
    while buffer.ends_with('\n') || buffer.ends_with('\r') {
        buffer.pop();
    }
    Ok(true)
}
