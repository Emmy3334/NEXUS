//! Tab completion for the current token.

mod collect;
mod context;
mod cycle;
mod docker;
mod docker_host;
mod git;
mod heal;
mod helm;
mod interp;
mod kube;
mod kubectl;
mod list;
mod paths;
mod subcmds;
mod systemctl;
mod token;
mod vars;

use self::collect::collect_matches;
use self::token::{apply_match, common_prefix, token_at};
use crate::env::ShellEnvironment;

pub use cycle::{complete_or_cycle, CompleteCycle};
pub use list::{format_columns, list_display_lines, list_display_lines_width, list_menu_lines};

/// Replace the token under the cursor; returns display lines for ambiguous matches.
pub fn complete(buffer: &mut String, cursor: &mut usize, env: &ShellEnvironment) -> Vec<String> {
    let names = env.var_names();
    complete_with_names(buffer, cursor, &names)
}

/// Like [`complete`], using a precomputed variable-name snapshot (TTY editor).
pub(crate) fn complete_with_names(
    buffer: &mut String,
    cursor: &mut usize,
    var_names: &[String],
) -> Vec<String> {
    let (start, prefix) = token_at(buffer, *cursor);
    let before = &buffer[..start];
    let matches = collect_matches(before, &prefix, var_names);
    match matches.as_slice() {
        [] => Vec::new(),
        [only] => {
            apply_match(buffer, cursor, start, only);
            Vec::new()
        }
        many => {
            if let Some(shared) = common_prefix(many) {
                if shared.len() > prefix.len() {
                    apply_match(buffer, cursor, start, &shared);
                }
            }
            many.to_vec()
        }
    }
}
