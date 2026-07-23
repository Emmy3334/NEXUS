//! “Did you mean?” hints after classic command-not-found.

mod collect;
mod rank;

use super::config::{self, Backend};
use crate::env::ShellEnvironment;

use std::io::{self, Write};

/// After `{cmd}: Command not found.`, print neighbors and an optional heal tip.
pub(super) fn write_after_not_found(
    program: &str,
    shell_env: &ShellEnvironment,
    healers_tried: bool,
    stderr: &mut impl Write,
) -> io::Result<()> {
    if program.is_empty() || config::quiet_from(shell_env) {
        return Ok(());
    }
    let suggestions = rank::top(program, &collect::candidates(shell_env));
    if !suggestions.is_empty() {
        writeln!(stderr, "nexus: did you mean: {} ?", suggestions.join(", "))?;
    }
    if healers_tried {
        let order = config::resolve(shell_env).order;
        writeln!(
            stderr,
            "nexus: tip: heal tried {} — set heal_order=… or NEXUS_LOG=nexus::heal=debug",
            order_csv(&order)
        )?;
    }
    Ok(())
}

fn order_csv(order: &[Backend]) -> String {
    order
        .iter()
        .map(|b| match b {
            Backend::Wasm => "wasm",
            Backend::Kube => "kube",
            Backend::Docker => "docker",
        })
        .collect::<Vec<_>>()
        .join(",")
}
