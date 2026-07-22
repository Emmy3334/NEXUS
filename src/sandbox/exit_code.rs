//! Map Wasmtime / WASI failures to shell exit statuses.

use wasmtime::Error;
use wasmtime_wasi::I32Exit;

use std::io::{self, Write};

pub(super) fn from_call_result(
    result: Result<(), Error>,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    match result {
        Ok(()) => Ok(0),
        Err(err) => {
            if let Some(exit) = err.downcast_ref::<I32Exit>() {
                return Ok(clamp_exit(exit.0));
            }
            writeln!(stderr, "sandbox: {err}")?;
            Ok(1)
        }
    }
}

fn clamp_exit(code: i32) -> u8 {
    u8::try_from(code).unwrap_or(1)
}
