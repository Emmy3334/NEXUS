//! Trivial status builtins: `true`, `false`, and `:`.

use crate::builtins::BuiltinResult;

use std::io::{self, Write};

pub(super) fn run_true(
    _argv: &[String],
    _stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    Ok(BuiltinResult::Status(0))
}

pub(super) fn run_false(
    _argv: &[String],
    _stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    Ok(BuiltinResult::Status(1))
}

pub(super) fn run_colon(
    _argv: &[String],
    _stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    Ok(BuiltinResult::Status(0))
}
