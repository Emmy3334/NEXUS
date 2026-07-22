//! `ignoreeof` — suppress interactive EOF until the threshold is hit.

use crate::env::ShellEnvironment;

use std::io::{self, Write};

/// Decide whether an interactive EOF should end the shell.
///
/// - Unset: exit immediately (`true`).
/// - Empty / `0`: keep prompting (`false`), print the leave hint.
/// - Number `n`: exit on the `n`th consecutive EOF.
pub fn allow_exit_on_eof(
    env: &ShellEnvironment,
    streak: &mut u32,
    stderr: &mut impl Write,
) -> io::Result<bool> {
    let Some(limit) = threshold(env) else {
        return Ok(true);
    };
    *streak = streak.saturating_add(1);
    if *streak >= limit {
        return Ok(true);
    }
    writeln!(stderr, "Use \"exit\" to leave nexus.")?;
    Ok(false)
}

fn threshold(env: &ShellEnvironment) -> Option<u32> {
    let value = env.get_local("ignoreeof")?;
    if value.is_empty() || value == "0" {
        return Some(u32::MAX);
    }
    match value.parse::<u32>() {
        Ok(0) => Some(u32::MAX),
        Ok(n) => Some(n),
        Err(_) => Some(u32::MAX),
    }
}
