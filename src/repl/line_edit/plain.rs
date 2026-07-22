//! Plain `BufRead` acquisition (pipes, scripts, and non-TTY interactive).

use std::io::{self, BufRead};

/// Read one physical line. `Ok(false)` is EOF; `Ok(true)` is a line (possibly empty).
pub(super) fn read_into(stdin: &mut impl BufRead, buffer: &mut String) -> io::Result<bool> {
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
