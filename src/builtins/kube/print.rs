//! Print kube tables / describe text with shared error handling.

use crate::builtins::BuiltinResult;
use crate::kube;

use std::io::{self, Write};

pub(super) fn table(
    result: io::Result<Vec<String>>,
    empty_scope: &str,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    match result {
        Ok(lines) if lines.is_empty() => {
            writeln!(stderr, "@kube: no resources in {empty_scope}")?;
            if empty_scope.contains("default") {
                writeln!(stderr, "try: @kube pods -A")?;
            }
            Ok(BuiltinResult::Status(0))
        }
        Ok(lines) => {
            for line in lines {
                writeln!(stdout, "{line}")?;
            }
            Ok(BuiltinResult::Status(0))
        }
        Err(err) => {
            writeln!(stderr, "@kube: {}", kube::format_err(&err))?;
            Ok(BuiltinResult::Status(1))
        }
    }
}

pub(super) fn nodes(stdout: &mut impl Write, stderr: &mut impl Write) -> io::Result<BuiltinResult> {
    table(kube::list_nodes_table(), "nodes", stdout, stderr)
}

pub(super) fn text(
    result: io::Result<String>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    match result {
        Ok(body) => {
            write!(stdout, "{body}")?;
            Ok(BuiltinResult::Status(0))
        }
        Err(err) => {
            writeln!(stderr, "@kube: {}", kube::format_err(&err))?;
            Ok(BuiltinResult::Status(1))
        }
    }
}
