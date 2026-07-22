//! Join physical lines until quotes are closed (interactive non-TTY).

use super::plain;
use super::probe::quotes_closed;
use super::ReplInput;
use crate::repl::prompt;

use std::collections::VecDeque;
use std::io::{self, Write};

pub(super) fn join_until_closed(
    stdin: &mut impl ReplInput,
    stdout: &mut impl Write,
    buffer: &mut String,
    queue: &mut VecDeque<u8>,
) -> io::Result<()> {
    let mut chunk = String::new();
    while !quotes_closed(buffer) {
        prompt::write_continue(stdout)?;
        chunk.clear();
        if !plain::read_into(stdin, &mut chunk, queue)? {
            // EOF mid-quote: leave buffer as-is for lex to report.
            break;
        }
        buffer.push('\n');
        buffer.push_str(&chunk);
    }
    Ok(())
}
