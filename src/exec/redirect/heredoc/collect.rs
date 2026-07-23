//! Walk the AST and read heredoc bodies from the shell input stream.

use crate::env::ShellEnvironment;
use crate::expand;
use crate::parse::{CommandList, PipelineCommand, RedirectKind};

use std::io::{self, BufRead, Write};

/// Read heredoc bodies for every `<<` in `list`, left-to-right (depth-first).
pub fn collect_heredoc_bodies(
    list: &CommandList<'_>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    input: &mut impl BufRead,
    stderr: &mut impl Write,
) -> io::Result<Result<Vec<String>, u8>> {
    let mut bodies = Vec::new();
    let mut line = String::new();
    match collect_from_list(
        list,
        shell_env,
        last_status,
        input,
        stderr,
        &mut bodies,
        &mut line,
    )? {
        Ok(()) => Ok(Ok(bodies)),
        Err(code) => Ok(Err(code)),
    }
}

fn collect_from_list(
    list: &CommandList<'_>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    input: &mut impl BufRead,
    stderr: &mut impl Write,
    bodies: &mut Vec<String>,
    line: &mut String,
) -> io::Result<Result<(), u8>> {
    for pipeline in &list.pipelines {
        for command in &pipeline.commands {
            if let Err(code) =
                collect_from_command(command, shell_env, last_status, input, stderr, bodies, line)?
            {
                return Ok(Err(code));
            }
        }
    }
    Ok(Ok(()))
}

fn collect_from_command(
    command: &PipelineCommand<'_>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    input: &mut impl BufRead,
    stderr: &mut impl Write,
    bodies: &mut Vec<String>,
    line: &mut String,
) -> io::Result<Result<(), u8>> {
    match command {
        PipelineCommand::Simple(simple) => collect_redirects(
            &simple.redirects,
            shell_env,
            last_status,
            input,
            stderr,
            bodies,
            line,
        ),
        PipelineCommand::Subshell { list, redirects } => {
            if let Err(code) = collect_redirects(
                redirects,
                shell_env,
                last_status,
                input,
                stderr,
                bodies,
                line,
            )? {
                return Ok(Err(code));
            }
            collect_from_list(list, shell_env, last_status, input, stderr, bodies, line)
        }
    }
}

fn collect_redirects(
    redirects: &[crate::parse::Redirect<'_>],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    input: &mut impl BufRead,
    stderr: &mut impl Write,
    bodies: &mut Vec<String>,
    line: &mut String,
) -> io::Result<Result<(), u8>> {
    for redirect in redirects {
        if redirect.kind != RedirectKind::Heredoc {
            continue;
        }
        let delimiter =
            match expand_heredoc_delimiter(redirect.path, shell_env, last_status, input, stderr)? {
                Ok(d) => d,
                Err(code) => return Ok(Err(code)),
            };
        bodies.push(read_heredoc_body(input, &delimiter, line, stderr)?);
    }
    Ok(Ok(()))
}

fn expand_heredoc_delimiter(
    raw: &str,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdin: &mut impl BufRead,
    stderr: &mut impl Write,
) -> io::Result<Result<String, u8>> {
    let mut fields = Vec::new();
    let snap = shell_env.clone();
    let mut capture =
        |body: &str| crate::exec::capture_command_output(body, &snap, last_status, stdin, stderr);
    match expand::expand_word_fields_into(raw, shell_env, last_status, &mut fields, &mut capture) {
        Ok(()) => Ok(Ok(fields
            .into_iter()
            .next()
            .unwrap_or_default()
            .into_string())),
        Err(err) => {
            writeln!(stderr, "{}", err.message())?;
            Ok(Err(1))
        }
    }
}

fn read_heredoc_body(
    input: &mut impl BufRead,
    delimiter: &str,
    line: &mut String,
    stderr: &mut impl Write,
) -> io::Result<String> {
    let mut body = String::new();
    loop {
        line.clear();
        let bytes = input.read_line(line)?;
        if bytes == 0 {
            writeln!(
                stderr,
                "nexus: warning: here-document delimited by end-of-file (wanted `{delimiter}`)"
            )?;
            break;
        }
        let without_ending = line.trim_end_matches(['\n', '\r']);
        if without_ending == delimiter {
            break;
        }
        body.push_str(line);
    }
    Ok(body)
}
