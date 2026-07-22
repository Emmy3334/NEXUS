//! Collect `if` then / else-if / else bodies until matching `endif`.

use super::line_edit::{self, ReadOutcome, ReplInput};
use super::ReplIo;
use crate::history::History;
use crate::if_block::{self, IfHeader};
use crate::keybind::KeyBindings;
use crate::lex;

use std::io::{self, Write};

/// Bodies collected for one top-level `if`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct IfPlan {
    pub(super) then_body: Vec<String>,
    pub(super) else_ifs: Vec<(IfHeader, Vec<String>)>,
    pub(super) else_body: Option<Vec<String>>,
}

/// Read until matching `endif`; `Ok(None)` means unexpected EOF.
pub(super) fn collect_if<I: ReplInput, O: Write, E: Write>(
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    history: &History,
    bindings: &mut KeyBindings,
) -> io::Result<Option<IfPlan>> {
    let mut plan = IfPlan {
        then_body: Vec::new(),
        else_ifs: Vec::new(),
        else_body: None,
    };
    let mut depth = 1_u32;
    let mut target = Target::Then;
    let mut line_buf = String::new();
    while depth > 0 {
        if !read_body_line(io, interactive, history, bindings, &mut line_buf)? {
            return Ok(None);
        }
        depth = ingest_line(depth, &line_buf, &mut plan, &mut target);
    }
    Ok(Some(plan))
}

fn read_body_line<I: ReplInput, O: Write, E: Write>(
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    history: &History,
    bindings: &mut KeyBindings,
    line_buf: &mut String,
) -> io::Result<bool> {
    if interactive {
        write!(io.stdout, "if? ")?;
        io.stdout.flush()?;
    }
    line_buf.clear();
    match line_edit::read_logical_line(
        io.stdin,
        io.stdout,
        false,
        history,
        bindings,
        line_buf,
        &mut io.input_queue,
    )? {
        ReadOutcome::Eof => {
            writeln!(io.stderr, "then: then/endif not found.")?;
            Ok(false)
        }
        ReadOutcome::Line => Ok(true),
    }
}

#[derive(Debug, Clone, Copy)]
enum Target {
    Then,
    ElseIf,
    Else,
}

fn ingest_line(depth: u32, line: &str, plan: &mut IfPlan, target: &mut Target) -> u32 {
    if if_block::line_opens_if(line) {
        push_line(plan, *target, line);
        return depth + 1;
    }
    if if_block::is_endif_line(line) {
        if depth > 1 {
            push_line(plan, *target, line);
        }
        return depth - 1;
    }
    if depth == 1 {
        if let Some(header) = parse_else_if_line(line) {
            *target = Target::ElseIf;
            plan.else_ifs.push((header, Vec::new()));
            return depth;
        }
        if if_block::is_else_line(line) {
            *target = Target::Else;
            plan.else_body = Some(Vec::new());
            return depth;
        }
    }
    push_line(plan, *target, line);
    depth
}

fn parse_else_if_line(line: &str) -> Option<IfHeader> {
    let mut tokens = Vec::new();
    lex::tokenize_into(line, &mut tokens).ok()?;
    if_block::parse_else_if(line, &tokens)
}

fn push_line(plan: &mut IfPlan, target: Target, line: &str) {
    match target {
        Target::Then => plan.then_body.push(line.to_owned()),
        Target::ElseIf => {
            if let Some((_, body)) = plan.else_ifs.last_mut() {
                body.push(line.to_owned());
            }
        }
        Target::Else => {
            if let Some(body) = plan.else_body.as_mut() {
                body.push(line.to_owned());
            }
        }
    }
}
