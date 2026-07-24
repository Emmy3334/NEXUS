//! Tab completion for the current token.

mod annotate;
mod aws;
mod collect;
mod context;
mod cycle;
mod docker;
mod docker_host;
mod engine;
mod gcloud;
mod git;
mod heal;
mod helm;
mod interp;
mod kube;
mod kubectl;
mod list;
mod list_tagged;
mod match_item;
mod matchers;
mod paths;
mod subcmds;
mod systemctl;
mod token;
mod vars;

use self::engine::run as run_engine;
use self::token::{apply_match, common_prefix, token_at};
use crate::env::{CompRegistry, ShellEnvironment};

/// Variable names + completion registry snapshot for one line read.
pub struct CompleteCtx<'a> {
    pub var_names: &'a [String],
    pub registry: &'a CompRegistry,
}

pub use cycle::{complete_or_cycle, CompleteCycle};
pub use list::{format_columns, list_display_lines, list_display_lines_width, list_menu_lines};
pub use list_tagged::list_menu_lines_tagged;
pub use match_item::{Match, Tag};

/// Ranked matches for a line with the cursor at the end (integration tests).
pub fn complete_matches_for_test(line: &str, env: &ShellEnvironment) -> Vec<Match> {
    let buffer = line.to_owned();
    let cursor = buffer.len();
    let (start, prefix) = token::token_at(&buffer, cursor);
    let before = &buffer[..start];
    let names = env.var_names();
    run_engine(before, &prefix, &names, &env.comp_registry)
}

/// Outcome of a single Tab completion attempt.
pub(crate) struct CompleteOutcome {
    pub listed: Vec<String>,
    pub matches: Vec<match_item::Match>,
}

/// Replace the token under the cursor; returns display lines for ambiguous matches.
pub fn complete(buffer: &mut String, cursor: &mut usize, env: &ShellEnvironment) -> Vec<String> {
    let names = env.var_names();
    let ctx = CompleteCtx {
        var_names: &names,
        registry: &env.comp_registry,
    };
    complete_with_ctx(buffer, cursor, &ctx).listed
}

/// Like [`complete`], using a precomputed snapshot (TTY editor).
pub(crate) fn complete_with_ctx(
    buffer: &mut String,
    cursor: &mut usize,
    ctx: &CompleteCtx<'_>,
) -> CompleteOutcome {
    let (start, prefix) = token_at(buffer, *cursor);
    let before = &buffer[..start];
    let matches = run_engine(before, &prefix, ctx.var_names, ctx.registry);
    let many = self::match_item::values(&matches);
    match many.as_slice() {
        [] => CompleteOutcome {
            listed: Vec::new(),
            matches,
        },
        [only] => {
            apply_match(buffer, cursor, start, only);
            CompleteOutcome {
                listed: Vec::new(),
                matches: Vec::new(),
            }
        }
        slice => {
            if let Some(shared) = common_prefix(slice) {
                if shared.len() > prefix.len() {
                    apply_match(buffer, cursor, start, &shared);
                }
            }
            CompleteOutcome {
                listed: slice.to_vec(),
                matches,
            }
        }
    }
}
