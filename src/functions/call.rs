//! Invoke a stored function with positional argv swap.

use crate::env::ShellEnvironment;
use crate::exec::{self, CommandResult};
use crate::lex;
use crate::parse;

use std::io::{self, Write};

const MAX_DEPTH: u32 = 64;

/// Run `argv[0]` as a function if defined; otherwise `Ok(None)`.
pub fn try_run(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<Option<CommandResult>> {
    let Some(name) = argv.first().map(String::as_str) else {
        return Ok(None);
    };
    let Some(body) = shell_env.function_get(name).map(str::to_owned) else {
        return Ok(None);
    };
    if shell_env.func_depth() >= MAX_DEPTH {
        writeln!(stderr, "{name}: function nesting too deep")?;
        return Ok(Some(CommandResult::Status(1)));
    }
    Ok(Some(invoke(
        name,
        &body,
        argv,
        shell_env,
        last_status,
        stdout,
        stderr,
    )?))
}

fn invoke(
    name: &str,
    body: &str,
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    let saved_argv = shell_env.argv().to_vec();
    let mut call_argv = Vec::with_capacity(argv.len());
    call_argv.push(name.to_owned());
    call_argv.extend(argv.iter().skip(1).cloned());
    shell_env.set_argv(call_argv);
    shell_env.enter_function();
    let result = run_body(body, shell_env, last_status, stdout, stderr);
    shell_env.leave_function();
    shell_env.set_argv(saved_argv);
    let _ = shell_env.take_return();
    result
}

fn run_body(
    body: &str,
    shell_env: &mut ShellEnvironment,
    mut last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    let mut argv = Vec::new();
    let mut tokens = Vec::new();
    let mut stdin = io::Cursor::new(Vec::<u8>::new());
    for raw in body.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        tokens.clear();
        if let Err(err) = lex::tokenize_into(line, &mut tokens) {
            writeln!(stderr, "{}", err.message())?;
            return Ok(CommandResult::Status(1));
        }
        match parse::parse_line(line, &tokens) {
            Ok(None) => {}
            Ok(Some(list)) => {
                match exec::execute_list_captured(
                    &list,
                    &mut argv,
                    shell_env,
                    last_status,
                    Vec::new(),
                    &mut stdin,
                    stdout,
                    stderr,
                )? {
                    CommandResult::Status(code) => {
                        last_status = code;
                        if shell_env.return_requested() {
                            return Ok(CommandResult::Status(
                                shell_env.take_return().unwrap_or(code),
                            ));
                        }
                    }
                    CommandResult::Exit(code) => return Ok(CommandResult::Exit(code)),
                    CommandResult::Source(_) => {
                        writeln!(stderr, "source: not supported in functions")?;
                        return Ok(CommandResult::Status(1));
                    }
                }
            }
            Err(err) => {
                writeln!(stderr, "{}", err.message())?;
                return Ok(CommandResult::Status(1));
            }
        }
    }
    Ok(CommandResult::Status(last_status))
}
