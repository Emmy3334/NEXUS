//! Collect `foreach` body lines until matching `end`.

use super::line_edit::{self, ReadOutcome, ReplInput};
use super::ReplIo;
use crate::foreach;
use crate::history::History;

use std::io::{self, Write};

/// Read body lines; `Ok(None)` means unexpected EOF (status 1).
pub(super) fn collect_body<I: ReplInput, O: Write, E: Write>(
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    history: &History,
) -> io::Result<Option<Vec<String>>> {
    let mut body = Vec::new();
    let mut depth = 1_u32;
    let mut line_buf = String::new();
    while depth > 0 {
        if interactive {
            write!(io.stdout, "foreach? ")?;
            io.stdout.flush()?;
        }
        line_buf.clear();
        match line_edit::read_logical_line(
            io.stdin,
            io.stdout,
            false,
            history,
            &mut line_buf,
            &mut io.input_queue,
        )? {
            ReadOutcome::Eof => {
                writeln!(io.stderr, "foreach: Unexpected end of file.")?;
                return Ok(None);
            }
            ReadOutcome::Line => {}
        }
        depth = update_depth(depth, &line_buf, &mut body);
    }
    Ok(Some(body))
}

fn update_depth(depth: u32, line: &str, body: &mut Vec<String>) -> u32 {
    if foreach::line_opens_foreach(line) {
        body.push(line.to_owned());
        depth + 1
    } else if foreach::is_end_line(line) {
        if depth > 1 {
            body.push(line.to_owned());
        }
        depth - 1
    } else {
        body.push(line.to_owned());
        depth
    }
}
