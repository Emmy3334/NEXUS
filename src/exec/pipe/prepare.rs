//! Expand and classify pipeline stages before spawning.

use crate::env::ShellEnvironment;
use crate::exec::io::ExecIo;
use crate::exec::CommandResult;
use crate::parse::{CommandList, Pipeline, PipelineCommand, Redirect};

use std::io::{self, BufRead, Write};

pub(super) enum PreparedStage<'a> {
    Simple {
        argv: Vec<String>,
        redirects: &'a [Redirect<'a>],
    },
    Subshell {
        list: &'a CommandList<'a>,
        redirects: &'a [Redirect<'a>],
    },
}

pub(super) fn build_stages<'a, I: BufRead, O: Write, E: Write>(
    pipeline: &'a Pipeline<'a>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<Result<Vec<PreparedStage<'a>>, CommandResult>> {
    let mut stages = Vec::with_capacity(pipeline.commands.len());
    let mut fields = Vec::new();
    for command in &pipeline.commands {
        match prepare_command(command, shell_env, last_status, io, &mut fields)? {
            Ok(stage) => stages.push(stage),
            Err(result) => return Ok(Err(result)),
        }
    }
    Ok(Ok(stages))
}

fn prepare_command<'a, I: BufRead, O: Write, E: Write>(
    command: &'a PipelineCommand<'a>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    io: &mut ExecIo<'_, I, O, E>,
    fields: &mut Vec<crate::expand::ExpandedWord>,
) -> io::Result<Result<PreparedStage<'a>, CommandResult>> {
    match command {
        PipelineCommand::Simple(simple) => {
            prepare_simple(simple, shell_env, last_status, io, fields)
        }
        PipelineCommand::Subshell { list, redirects } => Ok(Ok(PreparedStage::Subshell {
            list,
            redirects: redirects.as_slice(),
        })),
    }
}

fn prepare_simple<'a, I: BufRead, O: Write, E: Write>(
    simple: &'a crate::parse::SimpleCommand<'a>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    io: &mut ExecIo<'_, I, O, E>,
    fields: &mut Vec<crate::expand::ExpandedWord>,
) -> io::Result<Result<PreparedStage<'a>, CommandResult>> {
    let mut argv = Vec::with_capacity(simple.argv.len());
    for word in &simple.argv {
        if let Err(result) = expand_stage_word(word, shell_env, last_status, io, fields, &mut argv)?
        {
            return Ok(Err(result));
        }
    }
    if let Err(err) =
        crate::alias::apply_aliases(&mut argv, shell_env, last_status, io.stdin, io.stderr)
    {
        writeln!(io.stderr, "{}", err.message())?;
        return Ok(Err(CommandResult::Status(1)));
    }
    Ok(Ok(PreparedStage::Simple {
        argv,
        redirects: simple.redirects.as_slice(),
    }))
}

fn expand_stage_word<I: BufRead, O: Write, E: Write>(
    word: &str,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    io: &mut ExecIo<'_, I, O, E>,
    fields: &mut Vec<crate::expand::ExpandedWord>,
    argv: &mut Vec<String>,
) -> io::Result<Result<(), CommandResult>> {
    let result = if crate::expand::word_may_need_cmd_subst(word) {
        let snap = shell_env.clone_for_capture();
        let mut capture = |body: &str| {
            crate::exec::capture_command_output(body, &snap, last_status, io.stdin, io.stderr)
        };
        crate::expand::expand_word_fields_into(word, shell_env, last_status, fields, &mut capture)
    } else {
        let mut deny = |_: &str| Err(crate::lex::LexError::CommandSubstitution);
        crate::expand::expand_word_fields_into(word, shell_env, last_status, fields, &mut deny)
    };
    match result {
        Ok(()) => {
            for field in fields.drain(..) {
                argv.extend(crate::glob::expand_globs(&field));
            }
            Ok(Ok(()))
        }
        Err(err) => {
            writeln!(io.stderr, "{}", err.message())?;
            Ok(Err(CommandResult::Status(1)))
        }
    }
}
