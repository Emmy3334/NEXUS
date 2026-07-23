//! Detect foreach / while / if control headers after lex.

use super::line::ParseOutcome;
use crate::foreach;
use crate::if_block;
use crate::lex;
use crate::while_loop;

use std::io::{self, Write};

pub(super) fn try_control<'a, E: Write>(
    expanded: &'a str,
    tokens: &[lex::Token],
    stderr: &mut E,
) -> io::Result<Option<ParseOutcome<'a>>> {
    if let Some(outcome) = try_loop_control(expanded, tokens, stderr)? {
        return Ok(Some(outcome));
    }
    if let Some(header) = crate::functions::parse_header(expanded, tokens) {
        return Ok(Some(ParseOutcome::Function(header)));
    }
    if let Some(outcome) = try_case_control(expanded, tokens, stderr)? {
        return Ok(Some(outcome));
    }
    try_if_control(expanded, tokens, stderr)
}

fn try_loop_control<'a, E: Write>(
    expanded: &'a str,
    tokens: &[lex::Token],
    stderr: &mut E,
) -> io::Result<Option<ParseOutcome<'a>>> {
    if let Some(header) = foreach::parse_header(expanded, tokens) {
        return Ok(Some(ParseOutcome::ForEach(header)));
    }
    if foreach::starts_with_foreach(expanded, tokens) {
        writeln!(
            stderr,
            "{}",
            foreach::foreach_syntax_message(expanded, tokens)
        )?;
        return Ok(Some(ParseOutcome::Failed(1)));
    }
    if let Some(header) = while_loop::parse_header(expanded, tokens) {
        return Ok(Some(ParseOutcome::While(header)));
    }
    if while_loop::starts_with_while(expanded, tokens) {
        writeln!(
            stderr,
            "{}",
            while_loop::while_syntax_message(expanded, tokens)
        )?;
        return Ok(Some(ParseOutcome::Failed(1)));
    }
    if foreach::is_end_line(expanded) {
        writeln!(stderr, "end: Not in while/foreach.")?;
        return Ok(Some(ParseOutcome::Failed(1)));
    }
    Ok(None)
}

fn try_case_control<'a, E: Write>(
    expanded: &'a str,
    tokens: &[lex::Token],
    stderr: &mut E,
) -> io::Result<Option<ParseOutcome<'a>>> {
    if let Some(header) = crate::case_block::parse_header(expanded, tokens) {
        return Ok(Some(ParseOutcome::Case(header)));
    }
    if crate::case_block::starts_with_case(expanded, tokens) {
        writeln!(
            stderr,
            "{}",
            crate::case_block::case_syntax_message(expanded, tokens)
        )?;
        return Ok(Some(ParseOutcome::Failed(1)));
    }
    if crate::case_block::is_esac_line(expanded) {
        writeln!(stderr, "esac: Not in case.")?;
        return Ok(Some(ParseOutcome::Failed(1)));
    }
    Ok(None)
}

fn try_if_control<'a, E: Write>(
    expanded: &'a str,
    tokens: &[lex::Token],
    stderr: &mut E,
) -> io::Result<Option<ParseOutcome<'a>>> {
    if let Some(header) = if_block::parse_header(expanded, tokens) {
        return Ok(Some(ParseOutcome::If(header)));
    }
    if if_block::starts_with_if(expanded, tokens) {
        writeln!(stderr, "{}", if_block::if_syntax_message(expanded, tokens))?;
        return Ok(Some(ParseOutcome::Failed(1)));
    }
    if if_block::is_endif_line(expanded) {
        writeln!(stderr, "endif: Not in if.")?;
        return Ok(Some(ParseOutcome::Failed(1)));
    }
    if if_block::is_else_line(expanded) {
        writeln!(stderr, "else: Not in if.")?;
        return Ok(Some(ParseOutcome::Failed(1)));
    }
    Ok(None)
}
