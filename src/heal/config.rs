//! Heal chain settings: order, image, quiet (locals + env).

use crate::env::ShellEnvironment;

/// Named heal backend in the default chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Wasm,
    Kube,
    Docker,
}

/// Resolved heal settings for attaching backends.
#[derive(Debug, Clone)]
pub struct Settings {
    pub order: Vec<Backend>,
    pub image: String,
    pub quiet: bool,
    pub catch_all: bool,
    /// Display form of `heal_env` (`none` / `all` / `FOO,BAR`).
    pub env_pass: String,
}

const DEFAULT_IMAGE: &str = "alpine:3.20";
const DEFAULT_ORDER: &[Backend] = &[Backend::Wasm, Backend::Kube, Backend::Docker];

/// Resolve order / image / quiet / catch_all / env_pass from shell locals then process env.
#[must_use]
pub fn resolve(shell_env: &ShellEnvironment) -> Settings {
    Settings {
        order: order_from(shell_env),
        image: image_from(shell_env),
        quiet: quiet_from(shell_env),
        catch_all: super::image_map::catch_all(shell_env),
        env_pass: super::env_policy::EnvPass::from_shell(shell_env).label(),
    }
}

/// Parse a comma-separated backend list (`wasm`, `kube`, `docker`).
///
/// Unknown tokens are skipped; empty input yields the default order.
#[must_use]
pub fn parse_order(raw: &str) -> Vec<Backend> {
    let parsed: Vec<Backend> = raw
        .split(',')
        .filter_map(|tok| match tok.trim().to_ascii_lowercase().as_str() {
            "wasm" => Some(Backend::Wasm),
            "kube" | "kubernetes" | "k8s" => Some(Backend::Kube),
            "docker" => Some(Backend::Docker),
            _ => None,
        })
        .collect();
    if parsed.is_empty() {
        DEFAULT_ORDER.to_vec()
    } else {
        parsed
    }
}

#[must_use]
pub fn quiet_from(shell_env: &ShellEnvironment) -> bool {
    if let Some(v) = shell_env.lookup("heal_quiet") {
        return truthy(v);
    }
    std::env::var("NEXUS_HEAL_QUIET")
        .map(|v| truthy(&v))
        .unwrap_or(false)
}

fn order_from(shell_env: &ShellEnvironment) -> Vec<Backend> {
    if let Some(raw) = shell_env.lookup("heal_order") {
        return parse_order(raw);
    }
    match std::env::var("NEXUS_HEAL_ORDER") {
        Ok(raw) => parse_order(&raw),
        Err(_) => DEFAULT_ORDER.to_vec(),
    }
}

fn image_from(shell_env: &ShellEnvironment) -> String {
    if let Some(img) = shell_env.lookup("heal_image") {
        if !img.is_empty() {
            return img.to_owned();
        }
    }
    std::env::var("NEXUS_HEAL_IMAGE").unwrap_or_else(|_| DEFAULT_IMAGE.to_owned())
}

pub(crate) fn truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}
