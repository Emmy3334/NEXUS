//! Discoverability report for heal settings and backend reachability.

use super::config::{resolve, Backend, Settings};
use crate::env::ShellEnvironment;
use crate::kube;
use crate::sandbox;

/// Lines describing configured heal settings and live backend probes.
#[must_use]
pub fn status_lines(shell_env: &ShellEnvironment) -> Vec<String> {
    let settings = resolve(shell_env);
    let mut lines = Vec::with_capacity(5 + settings.order.len());
    push_settings(&settings, shell_env.healers.len(), &mut lines);
    for backend in &settings.order {
        lines.push(backend_line(*backend));
    }
    lines
}

fn push_settings(settings: &Settings, attached: usize, lines: &mut Vec<String>) {
    lines.push(format!("order: {}", order_csv(&settings.order)));
    lines.push(format!("image: {}", settings.image));
    lines.push(format!(
        "catch_all: {}",
        if settings.catch_all { "on" } else { "off" }
    ));
    lines.push(format!(
        "quiet: {}",
        if settings.quiet { "on" } else { "off" }
    ));
    lines.push(format!("session: {attached} attached"));
}

fn order_csv(order: &[Backend]) -> String {
    order
        .iter()
        .map(|b| backend_name(*b))
        .collect::<Vec<_>>()
        .join(",")
}

fn backend_name(backend: Backend) -> &'static str {
    match backend {
        Backend::Wasm => "wasm",
        Backend::Kube => "kube",
        Backend::Docker => "docker",
    }
}

fn backend_line(backend: Backend) -> String {
    match backend {
        Backend::Wasm => {
            let n = sandbox::list_names().len();
            format!("wasm: ready ({n} modules)")
        }
        Backend::Kube => {
            let state = if kube::cluster_reachable() {
                "ready"
            } else {
                "unavailable"
            };
            format!("kube: {state}")
        }
        Backend::Docker => {
            let state = if super::daemon_reachable() {
                "ready"
            } else {
                "unavailable"
            };
            format!("docker: {state}")
        }
    }
}
