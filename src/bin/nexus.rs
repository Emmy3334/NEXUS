//! NEXUS shell binary — thin entry point over the `nexus` library.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use nexus::repl;

const PROGRAM_FAILURE_EXIT: u8 = 84;

fn main() -> ExitCode {
    let interactive = io::stdin().is_terminal();

    match repl::run(io::stdin().lock(), io::stdout(), io::stderr(), interactive) {
        Ok(code) => ExitCode::from(code),
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(PROGRAM_FAILURE_EXIT)
        }
    }
}
