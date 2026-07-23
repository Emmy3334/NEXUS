//! Collect `case` arms until matching `esac`.

use super::line_edit::{self, ReadOutcome, ReplInput};
use super::ReplIo;
use crate::case_block;
use crate::history::History;
use crate::keybind::KeyBindings;

use std::io::{self, Write};

/// One `pat|pat) … ;;` arm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CaseArm {
    pub(super) patterns: Vec<String>,
    pub(super) body: Vec<String>,
}

/// All arms for one `case`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct CasePlan {
    pub(super) arms: Vec<CaseArm>,
}

/// Read until matching `esac`; `Ok(None)` means unexpected EOF.
pub(super) fn collect_case<I: ReplInput, O: Write, E: Write>(
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    history: &History,
    bindings: &mut KeyBindings,
) -> io::Result<Option<CasePlan>> {
    let mut plan = CasePlan::default();
    let mut depth = 1_u32;
    let mut in_body = false;
    let mut line_buf = String::new();
    while depth > 0 {
        if !read_body_line(io, interactive, history, bindings, &mut line_buf)? {
            return Ok(None);
        }
        depth = ingest_line(depth, &line_buf, &mut plan, &mut in_body);
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
        write!(io.stdout, "case? ")?;
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
        &[],
    )? {
        ReadOutcome::Eof => {
            writeln!(io.stderr, "case: Unexpected end of file.")?;
            Ok(false)
        }
        ReadOutcome::Line => Ok(true),
    }
}

fn ingest_line(depth: u32, line: &str, plan: &mut CasePlan, in_body: &mut bool) -> u32 {
    if case_block::line_opens_case(line) {
        if *in_body {
            push_body(plan, line);
        }
        return depth + 1;
    }
    if case_block::is_esac_line(line) {
        if depth > 1 {
            push_body(plan, line);
        }
        return depth - 1;
    }
    if depth != 1 {
        push_body(plan, line);
        return depth;
    }
    ingest_top(line, plan, in_body);
    depth
}

fn ingest_top(line: &str, plan: &mut CasePlan, in_body: &mut bool) {
    let trimmed = line.trim();
    if trimmed == ";;" {
        *in_body = false;
        return;
    }
    if !*in_body {
        if let Some(patterns) = parse_pattern_line(trimmed) {
            plan.arms.push(CaseArm {
                patterns,
                body: Vec::new(),
            });
            *in_body = true;
            return;
        }
    }
    push_body(plan, line);
}

fn parse_pattern_line(trimmed: &str) -> Option<Vec<String>> {
    let inner = trimmed.strip_suffix(')')?;
    if inner.is_empty() {
        return None;
    }
    Some(inner.split('|').map(|s| s.trim().to_owned()).collect())
}

fn push_body(plan: &mut CasePlan, line: &str) {
    if let Some(arm) = plan.arms.last_mut() {
        arm.body.push(line.to_owned());
    }
}
