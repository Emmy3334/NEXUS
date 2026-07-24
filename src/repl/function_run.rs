//! Collect and register a function definition.

use super::line_edit::{self, ReadOutcome, ReplInput};
use super::ReplIo;
use crate::env::ShellEnvironment;
use crate::exec::CommandResult;
use crate::functions::{body, FunctionHeader};

use std::io::{self, Write};

/// Define a function from a complete or open header.
pub(super) fn run_define<I: ReplInput, O: Write, E: Write>(
    header: FunctionHeader,
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    shell_env: &mut ShellEnvironment,
) -> io::Result<CommandResult> {
    match header {
        FunctionHeader::Complete { name, body } => {
            shell_env.function_set(name, body);
            Ok(CommandResult::Status(0))
        }
        FunctionHeader::Open { name, first } => {
            let Some(body) = collect_body(io, interactive, shell_env, first)? else {
                return Ok(CommandResult::Status(1));
            };
            shell_env.function_set(name, body);
            Ok(CommandResult::Status(0))
        }
    }
}

fn collect_body<I: ReplInput, O: Write, E: Write>(
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    shell_env: &mut ShellEnvironment,
    first: String,
) -> io::Result<Option<String>> {
    let mut lines = Vec::new();
    let mut depth = 1i32;
    if !first.is_empty() {
        depth += body::depth_delta(&first);
        lines.push(first);
    }
    let mut line_buf = String::new();
    while depth > 0 {
        if interactive {
            write!(io.stdout, "function> ")?;
            io.stdout.flush()?;
        }
        line_buf.clear();
        match line_edit::read_logical_line(
            io.stdin,
            io.stdout,
            false,
            &shell_env.history,
            &mut shell_env.key_bindings,
            &mut line_buf,
            &mut io.input_queue,
            None,
        )? {
            ReadOutcome::Eof => {
                writeln!(io.stderr, "function: Unexpected end of file.")?;
                return Ok(None);
            }
            ReadOutcome::Line => {}
        }
        depth += body::depth_delta(&line_buf);
        lines.push(std::mem::take(&mut line_buf));
    }
    Ok(Some(body::finalize_body(lines)))
}
