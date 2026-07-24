//! Collect `foreach` / `while` body lines until matching `end`.

use super::line_edit::{self, ReadOutcome, ReplInput};
use super::ReplIo;
use crate::foreach;
use crate::history::History;
use crate::keybind::KeyBindings;
use crate::while_loop;

use std::io::{self, Write};

/// Which block header started this body (controls prompt / EOF text).
#[derive(Debug, Clone, Copy)]
pub(super) enum BlockKind {
    ForEach,
    While,
}

/// Read body lines; `Ok(None)` means unexpected EOF (status 1).
pub(super) fn collect_body<I: ReplInput, O: Write, E: Write>(
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    history: &History,
    bindings: &mut KeyBindings,
    kind: BlockKind,
) -> io::Result<Option<Vec<String>>> {
    let mut body = Vec::new();
    let mut depth = 1_u32;
    let mut line_buf = String::new();
    while depth > 0 {
        if interactive {
            write!(io.stdout, "{} ", prompt(kind))?;
            io.stdout.flush()?;
        }
        line_buf.clear();
        match line_edit::read_logical_line(
            io.stdin,
            io.stdout,
            false,
            history,
            bindings,
            &mut line_buf,
            &mut io.input_queue,
            None,
            None,
        )? {
            ReadOutcome::Eof => {
                writeln!(io.stderr, "{}: Unexpected end of file.", keyword(kind))?;
                return Ok(None);
            }
            ReadOutcome::Line => {}
        }
        depth = update_depth(depth, &line_buf, &mut body);
    }
    Ok(Some(body))
}

fn prompt(kind: BlockKind) -> &'static str {
    match kind {
        BlockKind::ForEach => "foreach?",
        BlockKind::While => "while?",
    }
}

fn keyword(kind: BlockKind) -> &'static str {
    match kind {
        BlockKind::ForEach => "foreach",
        BlockKind::While => "while",
    }
}

fn update_depth(depth: u32, line: &str, body: &mut Vec<String>) -> u32 {
    if foreach::line_opens_foreach(line) || while_loop::line_opens_while(line) {
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
