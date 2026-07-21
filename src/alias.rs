//! Command-word alias expansion (after argv fill, before builtin/exec).

use crate::env::ShellEnvironment;
use crate::expand;
use crate::glob;
use crate::lex::{self, LexError, TokenKind};

use std::collections::HashSet;

/// Expand `argv[0]` through aliases until it is not an alias (or a cycle).
///
/// Does not expand when the command is `alias` or `unalias` (so those builtins
/// remain reachable). Alias bodies are lexed as words, then `$`/glob expanded.
pub fn apply_aliases(
    argv: &mut Vec<String>,
    shell_env: &ShellEnvironment,
    last_status: u8,
) -> Result<(), LexError> {
    let mut seen = HashSet::new();
    loop {
        let Some(cmd) = argv.first().map(String::as_str) else {
            return Ok(());
        };
        if matches!(cmd, "alias" | "unalias") {
            return Ok(());
        }
        if !seen.insert(cmd.to_owned()) {
            // Recursive alias — stop and leave the name as-is.
            return Ok(());
        }
        let Some(body) = shell_env.alias_get(cmd) else {
            return Ok(());
        };
        let words = expand_alias_body(body, shell_env, last_status)?;
        replace_command_word(argv, words);
    }
}

fn expand_alias_body(
    body: &str,
    shell_env: &ShellEnvironment,
    last_status: u8,
) -> Result<Vec<String>, LexError> {
    let mut tokens = Vec::new();
    lex::tokenize_into(body, &mut tokens)?;
    let mut words = Vec::new();
    for token in &tokens {
        if token.kind != TokenKind::Word {
            // Operators in alias bodies are out of scope for this slice.
            continue;
        }
        let raw = token.lexeme(body);
        let expanded = expand::expand_word_for_exec(raw, shell_env, last_status)?;
        words.extend(glob::expand_globs(&expanded));
    }
    Ok(words)
}

fn replace_command_word(argv: &mut Vec<String>, words: Vec<String>) {
    let rest: Vec<String> = argv.drain(1..).collect();
    argv.clear();
    if words.is_empty() {
        argv.extend(rest);
        return;
    }
    argv.extend(words);
    argv.extend(rest);
}
