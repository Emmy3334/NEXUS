//! Syntax analysis for command lists and pipelines (Dragon Book Ch. 4).
//!
//! Grammar (42sh parentheses + job control + and-or):
//! ```text
//! line     → list
//! list     → pipeline ( (';' | '&' | '&&' | '||') pipeline )* [ ';' | '&' ]
//! pipeline → command ( '|' command )*
//! command  → simple | '(' list ')' redirect*
//! simple   → ( WORD | redirect )+   # at least one WORD
//! redirect → ( '>' | '<' | '>>' | '<<' ) WORD
//! ```
//!
//! Heredoc body lines are collected by the REPL after parse (not on this line).
//! Empty commands around `|` / `&&` / `||` are errors; a trailing `;` or `&` is allowed.

mod parser;

use crate::lex::Token;

/// A sequence of pipelines separated by `;`, `&`, `&&`, or `||`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandList<'a> {
    pub pipelines: Vec<Pipeline<'a>>,
}

impl<'a> CommandList<'a> {
    /// A list that is exactly one foreground simple command (no `;`, `|`, `&`, `&&`, `||`, or subshell).
    #[must_use]
    pub fn as_single_command(&self) -> Option<&SimpleCommand<'a>> {
        match self.pipelines.as_slice() {
            [pipeline] if !pipeline.background && pipeline.join == PipelineJoin::Seq => {
                match pipeline.commands.as_slice() {
                    [PipelineCommand::Simple(cmd)] => Some(cmd),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

/// How this pipeline connects to the previous one in the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineJoin {
    /// First pipeline, or after `;` / `&`.
    Seq,
    /// After `&&` — run only if the previous status was zero.
    And,
    /// After `||` — run only if the previous status was non-zero.
    Or,
}

/// One or more commands connected by `|`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pipeline<'a> {
    pub commands: Vec<PipelineCommand<'a>>,
    /// True when this pipeline was terminated with `&`.
    pub background: bool,
    /// How this pipeline joins from the previous list element.
    pub join: PipelineJoin,
}

/// One stage of a pipeline: a simple command or a `( … )` subshell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineCommand<'a> {
    Simple(SimpleCommand<'a>),
    Subshell {
        list: CommandList<'a>,
        redirects: Vec<Redirect<'a>>,
    },
}

impl<'a> PipelineCommand<'a> {
    /// Redirects attached to this stage (simple or group).
    #[must_use]
    pub fn redirects(&self) -> &[Redirect<'a>] {
        match self {
            Self::Simple(cmd) => &cmd.redirects,
            Self::Subshell { redirects, .. } => redirects,
        }
    }
}

/// A simple command: argv plus optional file / heredoc redirections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleCommand<'a> {
    pub argv: Vec<&'a str>,
    pub redirects: Vec<Redirect<'a>>,
}

/// One redirection attached to a simple command or subshell group.
///
/// For file redirects, [`Redirect::path`] is the file path. For heredoc,
/// it is the end delimiter (body is collected later from the input stream).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirect<'a> {
    pub kind: RedirectKind,
    pub path: &'a str,
}

/// Redirection operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectKind {
    /// `<` — stdin from file.
    Read,
    /// `>` — stdout truncate/create.
    Write,
    /// `>>` — stdout append.
    Append,
    /// `<<` — stdin from heredoc body until `path` delimiter line.
    Heredoc,
}

/// Why `parse_line` failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// Missing command where one is required (e.g. leading/trailing `|`).
    NullCommand,
    /// Redirect operator without a following word.
    MissingRedirectTarget,
    /// Token that cannot start or continue the current construct.
    UnexpectedToken,
}

impl ParseError {
    /// Human-readable message for stderr.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::NullCommand => "Invalid null command.",
            Self::MissingRedirectTarget => "Missing name for redirect.",
            Self::UnexpectedToken => "Syntax error.",
        }
    }
}

/// Parse `tokens` spanning `source` into a command list.
///
/// Returns `Ok(None)` when there is nothing to run (empty input or only `;`).
pub fn parse_line<'a>(
    source: &'a str,
    tokens: &[Token],
) -> Result<Option<CommandList<'a>>, ParseError> {
    parser::Parser::new(source, tokens).parse_list()
}

/// Fill `argv` from raw word lexemes: quotes/`$`/`` ` `` expansion, then globs.
///
/// One raw word may expand to multiple argv entries. Reuses `argv` capacity
/// when the result fits in existing slots.
pub fn fill_argv(
    words: &[&str],
    argv: &mut Vec<String>,
    env: &mut crate::env::ShellEnvironment,
    last_status: u8,
    stdin: &mut impl std::io::BufRead,
    stderr: &mut impl std::io::Write,
) -> Result<(), crate::lex::LexError> {
    let mut expanded = Vec::new();
    let mut fields = Vec::new();
    for raw in words {
        expand_word_into(raw, env, last_status, stdin, stderr, &mut fields)?;
        for field in fields.drain(..) {
            expanded.extend(crate::glob::expand_globs(&field));
        }
    }

    while argv.len() < expanded.len() {
        argv.push(String::new());
    }
    argv.truncate(expanded.len());
    for (slot, value) in argv.iter_mut().zip(expanded) {
        *slot = value;
    }
    Ok(())
}

fn expand_word_into(
    raw: &str,
    env: &mut crate::env::ShellEnvironment,
    last_status: u8,
    stdin: &mut impl std::io::BufRead,
    stderr: &mut impl std::io::Write,
    fields: &mut Vec<crate::expand::ExpandedWord>,
) -> Result<(), crate::lex::LexError> {
    if crate::expand::word_may_need_cmd_subst(raw) {
        let snap = env.clone_for_capture();
        let mut capture = |body: &str| {
            crate::exec::capture_command_output(body, &snap, last_status, stdin, stderr)
        };
        crate::expand::expand_word_fields_into(raw, env, last_status, fields, &mut capture)
    } else {
        let mut deny = |_: &str| Err(crate::lex::LexError::CommandSubstitution);
        crate::expand::expand_word_fields_into(raw, env, last_status, fields, &mut deny)
    }
}
