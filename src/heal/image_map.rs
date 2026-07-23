//! Command → image map for Docker/Kube heal (mapped cmds only by default).

use crate::env::ShellEnvironment;

use super::config;

/// Small built-in map: tools alpine lacks → official alpine-tagged images.
const MAP: &[(&str, &str)] = &[
    ("python", "python:3.12-alpine"),
    ("python3", "python:3.12-alpine"),
    ("node", "node:22-alpine"),
    ("npm", "node:22-alpine"),
    ("npx", "node:22-alpine"),
    ("ruby", "ruby:3.3-alpine"),
    ("php", "php:cli-alpine"),
];

/// Alpine catch-all for unmapped commands (`heal_catch_all` / `NEXUS_HEAL_CATCH_ALL`).
///
/// Default is off so typos stay fast; mapped commands still heal.
#[must_use]
pub fn catch_all(shell_env: &ShellEnvironment) -> bool {
    if let Some(v) = shell_env.lookup("heal_catch_all") {
        return config::truthy(v);
    }
    std::env::var("NEXUS_HEAL_CATCH_ALL")
        .map(|v| config::truthy(&v))
        .unwrap_or(false)
}

/// Image for `cmd`: mapped specialty image, otherwise `base`.
#[must_use]
pub fn image_for<'a>(cmd: &str, base: &'a str) -> &'a str {
    lookup(cmd).unwrap_or(base)
}

/// Docker/Kube may run only for mapped commands, unless catch-all is on.
#[must_use]
pub fn container_heal_allowed(cmd: &str, shell_env: &ShellEnvironment) -> bool {
    lookup(cmd).is_some() || catch_all(shell_env)
}

fn lookup(cmd: &str) -> Option<&'static str> {
    let name = cmd.rsplit('/').next().unwrap_or(cmd);
    MAP.iter()
        .find(|(c, _)| *c == name)
        .map(|(_, image)| *image)
}
