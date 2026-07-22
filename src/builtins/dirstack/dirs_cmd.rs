//! `dirs` — print, clear, or save/load the directory stack.

use super::args::{take_print_flags, PrintFlags};
use super::file_ops::{load_stack, save_stack};
use super::print::print_stack;
use crate::env::ShellEnvironment;

use std::env as process_env;
use std::io::{self, Write};
use std::path::PathBuf;

enum DirsAction {
    Print(PrintFlags),
    Clear,
    Save(Option<PathBuf>),
    Load(Option<PathBuf>),
}

pub(crate) fn dirs_cmd(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let cwd = process_env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    shell_env.dir_stack.ensure_seeded(cwd.clone());
    match parse_argv(argv) {
        Ok(DirsAction::Print(flags)) => print_stack(shell_env, flags, stdout),
        Ok(DirsAction::Clear) => {
            shell_env.dir_stack.reset_to(cwd);
            Ok(0)
        }
        Ok(DirsAction::Save(path)) => save_stack(shell_env, path.as_deref(), stderr),
        Ok(DirsAction::Load(path)) => {
            load_stack(shell_env, path.as_deref(), last_status, stdout, stderr)
        }
        Err(msg) => {
            writeln!(stderr, "{msg}")?;
            Ok(1)
        }
    }
}

fn parse_argv(argv: &[String]) -> Result<DirsAction, String> {
    let args = &argv[1..];
    if args.is_empty() {
        return Ok(DirsAction::Print(PrintFlags::default()));
    }
    match args[0].as_str() {
        "-c" => {
            if args.len() > 1 {
                return Err(usage());
            }
            Ok(DirsAction::Clear)
        }
        "-S" | "-L" => parse_file_op(args),
        _ => parse_print(args),
    }
}

fn parse_file_op(args: &[String]) -> Result<DirsAction, String> {
    if args.len() > 2 {
        return Err(usage());
    }
    let path = args.get(1).map(PathBuf::from);
    Ok(match args[0].as_str() {
        "-S" => DirsAction::Save(path),
        _ => DirsAction::Load(path),
    })
}

fn parse_print(args: &[String]) -> Result<DirsAction, String> {
    let (flags, rest) = take_print_flags(args).map_err(|_| usage())?;
    if !rest.is_empty() {
        return Err(usage());
    }
    Ok(DirsAction::Print(flags))
}

fn usage() -> String {
    "Usage: dirs [-plvnSLc].".into()
}
