//! NEXUS shell binary — thin entry point over the `nexus` library.

use std::env;
use std::io::{self, IsTerminal};
use std::path::Path;
use std::process::ExitCode;

use nexus::env::ShellEnvironment;
use nexus::observability;
use nexus::repl;

const PROGRAM_FAILURE_EXIT: u8 = 84;

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(PROGRAM_FAILURE_EXIT)
        }
    }
}

fn run() -> io::Result<u8> {
    observability::init();
    let mut args: Vec<String> = env::args().collect();
    let program = args.first().cloned().unwrap_or_else(|| "nexus".into());
    if args.len() >= 2 {
        return run_script_args(&mut args);
    }
    let interactive = io::stdin().is_terminal();
    let mut shell_env = ShellEnvironment::capture();
    shell_env.set_argv(vec![program]);
    repl::run_with_env(
        io::stdin().lock(),
        io::stdout(),
        io::stderr(),
        interactive,
        &mut shell_env,
    )
}

fn run_script_args(args: &mut Vec<String>) -> io::Result<u8> {
    let script = args.remove(1);
    if !Path::new(&script).is_file() {
        eprintln!("{script}: No such file.");
        return Ok(1);
    }
    let mut argv = vec![script.clone()];
    argv.extend(args.iter().skip(1).cloned());
    repl::run_script(script, argv, io::stdout(), io::stderr())
}
