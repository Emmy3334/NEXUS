mod repl;

use std::io::{self, IsTerminal};
use std::process::ExitCode;

const PROGRAM_FAILURE_EXIT: u8 = 84;

fn main() -> ExitCode {
    let interactive = io::stdin().is_terminal();

    match repl::run(io::stdin().lock(), io::stdout(), interactive) {
        Ok(code) => ExitCode::from(code),
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(PROGRAM_FAILURE_EXIT)
        }
    }
}
